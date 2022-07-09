//! Read, Execute, Print Loop functionality
use std::sync::{RwLock};
use std::sync::mpsc::{self, Receiver};
use std::io::{self, Write};

use lazy_static::lazy_static;

use console::{Key, Term};

use muss_interpreter::{Interpreter, Debugger, Item, InterpreterEvent, InterpreterError};
use muss_interpreter::lang::TypePrimitive;
use muss_player::{Controller, Player};

use super::channel_io::{channel_io, ChannelWriter};
use super::cli::CliArgs;

lazy_static! {
    static ref DEBUG_STATE: RwLock<DebugState> = RwLock::new(
        DebugState {
            debug_flag: DebugFlag::Normal,
            verbose: false,
        }
    );
}

const TERMINAL_WRITE_ERROR: &str = "Failed to write to terminal output";
const INTERPRETER_WRITE_ERROR: &str = "Failed to write to interpreter";

type DebugItem = Result<Item, String>;

struct ReplState {
    terminal: Term,
    line_number: usize,
    statement_buf: Vec<char>,
    writer: ChannelWriter,
    in_literal: Option<char>,
    bracket_depth: usize,
    curly_depth: usize,
    history: Vec<String>,
    selected_history: usize,
    current_line: Vec<char>,
    cursor_rightward_position: usize,
    //debug: Arc<RwLock<DebugState>>,
    list_rx: Receiver<DebugItem>,
}

#[derive(Clone)]
struct DebugState {
    debug_flag: DebugFlag,
    verbose: bool,
}

#[derive(Copy, Clone)]
enum DebugFlag {
    Skip,
    List,
    Normal
}

impl ReplState {
    fn new(chan_writer: ChannelWriter, term: Term, debug_list: Receiver<DebugItem>) -> Self {
        Self {
            terminal: term,
            line_number: 0,
            statement_buf: Vec::new(),
            writer: chan_writer,
            in_literal: None,
            bracket_depth: 0,
            curly_depth: 0,
            history: Vec::new(),
            selected_history: 0,
            current_line: Vec::new(),
            cursor_rightward_position: 0,
            /*debug: Arc::new(RwLock::new(DebugState {
                debug_flag: DebugFlag::Normal,
            })),*/
            list_rx: debug_list,
        }
    }
}

fn interpreter_event_callback<'a, T: muss_interpreter::tokens::TokenReader>(_interpreter: &mut Interpreter<'a, T>, event: InterpreterEvent) -> Result<(), InterpreterError> {
    match event {
        InterpreterEvent::StatementComplete => {
            if let Ok(mut d_state) = DEBUG_STATE.write() {
                d_state.debug_flag = DebugFlag::Normal;
            }
        },
        _ => {},
    }
    Ok(())
}

fn pretty_print_item(item: &Item, terminal: &mut Term, args: &CliArgs, verbose: bool) {
    if verbose {
        writeln!(terminal, "I{}--\\/-- `{}` --\\/--", args.prompt,
            item.field("title").unwrap_or(&TypePrimitive::Empty).as_str()
        ).expect(TERMINAL_WRITE_ERROR);
        let mut fields: Vec<&_> = item.iter().collect();
        fields.sort();
        for field in fields {
            if field != "title" {
                writeln!(terminal, "I{}  {}: `{}`",
                    args.prompt, field,
                    item.field(field).unwrap_or(&TypePrimitive::Empty).as_str()
                ).expect(TERMINAL_WRITE_ERROR);
            }
        }
    } else {
         writeln!(terminal, "I{}`{}` by `{}`", args.prompt,
            item.field("title").unwrap_or(&TypePrimitive::Empty).as_str(),
            item.field("artist").unwrap_or(&TypePrimitive::Empty).as_str(),
        ).expect(TERMINAL_WRITE_ERROR);
    }
    //writeln!(terminal, "I{}----", args.prompt).expect(TERMINAL_WRITE_ERROR);
}

fn handle_list_rx(state: &mut ReplState, args: &CliArgs) {
    //let items = state.list_rx.try_iter().collect::<Vec<_>>();
    let d_state = DEBUG_STATE.read().expect("Failed to get read lock for debug state info").clone();
    for item in state.list_rx.try_iter() {
        match item {
            Ok(item) => pretty_print_item(&item, &mut state.terminal, args, d_state.verbose),
            Err(e) => error_prompt(
                muss_player::PlayerError::Playback(
                    muss_player::PlaybackError::from_err(e)
                ), args),
        }
    }
    let flag = d_state.debug_flag;
    match flag {
        DebugFlag::List => {
            while let Ok(item) = state.list_rx.recv() {
                match item {
                    Ok(item) => pretty_print_item(&item, &mut state.terminal, args, d_state.verbose),
                    Err(e) => error_prompt(
                        muss_player::PlayerError::Playback(
                            muss_player::PlaybackError::from_err(e)
                        ), args),
                }
                // stop listing if no longer in list mode
                let flag = if let Ok(d_state) = DEBUG_STATE.read() {
                    d_state.debug_flag
                } else {
                    DebugFlag::Normal
                };
                match flag {
                    DebugFlag::List => {},
                    _ => break,
                }
            }
        },
        _ => {}
    }
}

pub fn repl(args: CliArgs) {
    let term = Term::stdout();
    term.set_title("muss");
    let (writer, reader) = channel_io();
    let volume = args.volume.clone();
    let mpd = match args.mpd.clone().map(|a| muss_player::mpd_connection(a.parse().unwrap())).transpose() {
        Ok(mpd) => mpd,
        Err(e) => {
            eprintln!("Cannot connect to MPD address `{}`: {}", args.mpd.unwrap(), e);
            return;
        }
    };
    let (list_tx, list_rx) = mpsc::channel();
    let mut state = ReplState::new(writer, term, list_rx);
    let player_builder = move || {
        let runner = Interpreter::with_stream_and_callback(reader,
            &interpreter_event_callback);
        let debugger = Debugger::new(runner, move |interpretor, item| {
            let flag = if let Ok(d_state) = DEBUG_STATE.read() {
                d_state.debug_flag
            } else {
                DebugFlag::Normal
            };
            match flag {
                DebugFlag::Normal => item,
                DebugFlag::Skip => {
                    while let Some(_) = interpretor.next() {
                        // NOTE: recursion occurs here
                    }
                    None
                },
                DebugFlag::List => {
                    if let Some(x) = item {
                        list_tx.send(x.map_err(|e| e.to_string())).unwrap_or(());
                        while let Some(x) = interpretor.next() {
                            // NOTE: recursion occurs here
                            // in most cases this will never be a case of Some(...) because
                            // recursive calls to this function intercept it first and return None
                            list_tx.send(x.map_err(|e| e.to_string())).unwrap_or(());
                        }
                    }
                    None
                }
            }
        });

        let mut player = Player::new(debugger).unwrap();
        if let Some(vol) = volume {
            player.set_volume(vol);
        }
        if let Some(mpd) = mpd {
            player.set_mpd(mpd);
        }
        player
    };
    if let Some(playlist_file) = &args.playlist {
        if args.mpd.is_some() {
            writeln!(
                state.terminal,
                "Playlist mode (output: `{}` & MPD)",
                playlist_file
            )
            .expect(TERMINAL_WRITE_ERROR);
        } else {
            writeln!(
                state.terminal,
                "Playlist mode (output: `{}`)",
                playlist_file
            )
            .expect(TERMINAL_WRITE_ERROR);
        }
        let mut player = player_builder();
        let mut playlist_writer =
            io::BufWriter::new(std::fs::File::create(playlist_file).unwrap_or_else(|_| {
                panic!("Abort: Cannot create writeable file `{}`", playlist_file)
            }));
        read_loop(&args, &mut state, |state, args| {
            match player.save_m3u8(&mut playlist_writer) {
                Ok(_) => {}
                Err(e) => {
                    error_prompt(e, &args);
                    // consume any further errors (this shouldn't actually write anything)
                    while let Err(e) = player.save_m3u8(&mut playlist_writer) {
                        error_prompt(e, &args);
                    }
                }
            }
            playlist_writer
                .flush()
                .expect("Failed to flush playlist to file");
            handle_list_rx(state, args);
        });
    } else {
        if args.mpd.is_some() {
            writeln!(state.terminal, "Playback mode (output: audio device & MPD)")
                .expect(TERMINAL_WRITE_ERROR);
        } else {
            writeln!(state.terminal, "Playback mode (output: audio device)")
                .expect(TERMINAL_WRITE_ERROR);
        }
        let ctrl = Controller::create_repl(player_builder);
        read_loop(&args, &mut state, |state, args| {
            if args.wait {
                match ctrl.wait_for_empty() {
                    Ok(_) => {}
                    Err(e) => error_prompt(e, &args),
                }
            } else {
                // consume all incoming errors
                let mut had_err = true;
                while had_err {
                    let mut new_had_err = false;
                    for e in ctrl.check_ack() {
                        error_prompt(e, &args);
                        new_had_err = true;
                    }
                    had_err = new_had_err;
                }
            }
            handle_list_rx(state, args);
        });
    }
}

fn read_loop<F: FnMut(&mut ReplState, &CliArgs)>(args: &CliArgs, state: &mut ReplState, mut execute: F) -> ! {
    prompt(state, args);
    loop {
        match state
            .terminal
            .read_key()
            .expect("Failed to read terminal input")
        {
            Key::Char(read_c) => {
                if state.cursor_rightward_position == 0 {
                    write!(state.terminal, "{}", read_c)
                        .expect(TERMINAL_WRITE_ERROR);
                    state.statement_buf.push(read_c);
                    state.current_line.push(read_c);
                } else {
                    write!(state.terminal, "{}", read_c)
                        .expect(TERMINAL_WRITE_ERROR);
                    for i in state.current_line.len() - state.cursor_rightward_position
                        ..state.current_line.len()
                    {
                        write!(state.terminal, "{}", state.current_line[i])
                            .expect(TERMINAL_WRITE_ERROR);
                    }
                    state
                        .terminal
                        .move_cursor_left(state.cursor_rightward_position)
                        .expect(TERMINAL_WRITE_ERROR);
                    state.statement_buf.insert(
                        state.statement_buf.len() - state.cursor_rightward_position,
                        read_c,
                    );
                    state.current_line.insert(
                        state.current_line.len() - state.cursor_rightward_position,
                        read_c,
                    );
                }
                match read_c {
                    '"' | '`' => {
                        if let Some(c) = state.in_literal {
                            if c == read_c {
                                state.in_literal = None;
                            }
                        } else {
                            state.in_literal = Some(read_c);
                        }
                    }
                    '(' => state.bracket_depth += 1,
                    ')' => {
                        if state.bracket_depth != 0 {
                            state.bracket_depth -= 1
                        }
                    }
                    '{' => state.curly_depth += 1,
                    '}' => {
                        if state.curly_depth != 0 {
                            state.curly_depth -= 1
                        }
                    }
                    ';' => {
                        if state.in_literal.is_none() {
                            let statement = state.statement_buf.iter().collect::<String>();
                            let statement_result = statement.trim();
                            if !statement_result.starts_with('?') {
                                state
                                    .writer
                                    .write(state.statement_buf.iter().collect::<String>().as_bytes())
                                    .expect(INTERPRETER_WRITE_ERROR);
                                execute(state, args);
                                state.statement_buf.clear();
                            }
                        }
                    }
                    '\n' => {
                        let statement = state.statement_buf.iter().collect::<String>();
                        let statement_result = statement.trim();
                        if statement_result.starts_with('?') {
                            //println!("Got {}", statement_result.unwrap());
                            repl_commands(statement_result, state, args);
                            state.statement_buf.clear();
                        } else if state.bracket_depth == 0
                            && state.in_literal.is_none()
                            && state.curly_depth == 0
                        {
                            state.statement_buf.push(';');
                            state
                                .writer
                                .write(state.statement_buf.iter().collect::<String>().as_bytes())
                                .expect(INTERPRETER_WRITE_ERROR);
                            execute(state, args);
                            state.statement_buf.clear();
                        }
                        prompt(state, args);
                    }
                    _ => {}
                }
            }
            Key::Backspace => {
                if state.cursor_rightward_position == 0 {
                    if let Some(c) = state.statement_buf.pop() {
                        // re-sync syntax tracking
                        match c {
                            '"' | '`' => {
                                if let Some(c2) = state.in_literal {
                                    if c == c2 {
                                        state.in_literal = None;
                                    }
                                } else {
                                    state.in_literal = Some(c);
                                }
                            }
                            '(' => {
                                if state.bracket_depth != 0 {
                                    state.bracket_depth -= 1
                                }
                            }
                            ')' => state.bracket_depth += 1,
                            '{' => {
                                if state.curly_depth != 0 {
                                    state.curly_depth -= 1
                                }
                            }
                            '}' => state.curly_depth += 1,
                            _ => {}
                        }
                        match c {
                            '\n' | '\r' => {
                                // another line, cannot backspace that far
                                state.statement_buf.push(c);
                            }
                            _ => {
                                state.current_line.pop();
                                state
                                    .terminal
                                    .move_cursor_left(1)
                                    .expect(TERMINAL_WRITE_ERROR);
                                write!(state.terminal, " ")
                                    .expect(TERMINAL_WRITE_ERROR);
                                state
                                    .terminal
                                    .flush()
                                    .expect("Failed to flush terminal output");
                                state
                                    .terminal
                                    .move_cursor_left(1)
                                    .expect(TERMINAL_WRITE_ERROR);
                            }
                        }
                    }
                } else {
                    if state.current_line.len() != state.cursor_rightward_position {
                        // if not at start of line
                        let removed_char = state
                            .current_line
                            .remove(state.current_line.len() - state.cursor_rightward_position - 1);
                        state.statement_buf.remove(
                            state.statement_buf.len() - state.cursor_rightward_position - 1,
                        );
                        // re-sync unclosed syntax tracking
                        match removed_char {
                            '"' | '`' => {
                                if let Some(c2) = state.in_literal {
                                    if removed_char == c2 {
                                        state.in_literal = None;
                                    }
                                } else {
                                    state.in_literal = Some(removed_char);
                                }
                            }
                            '(' => {
                                if state.bracket_depth != 0 {
                                    state.bracket_depth -= 1
                                }
                            }
                            ')' => state.bracket_depth += 1,
                            '{' => {
                                if state.curly_depth != 0 {
                                    state.curly_depth -= 1
                                }
                            }
                            '}' => state.curly_depth += 1,
                            _ => {}
                        }
                        // re-print end of line to remove character in middle
                        state
                            .terminal
                            .move_cursor_left(1)
                            .expect(TERMINAL_WRITE_ERROR);
                        for i in state.current_line.len() - state.cursor_rightward_position
                            ..state.current_line.len()
                        {
                            write!(state.terminal, "{}", state.current_line[i])
                                .expect(TERMINAL_WRITE_ERROR);
                        }
                        write!(state.terminal, " ").expect(TERMINAL_WRITE_ERROR);
                        state
                            .terminal
                            .move_cursor_left(state.cursor_rightward_position + 1)
                            .expect(TERMINAL_WRITE_ERROR);
                    }
                }
            }
            Key::Enter => {
                state
                    .terminal
                    .write_line("")
                    .expect(TERMINAL_WRITE_ERROR);
                let statement = state.statement_buf.iter().collect::<String>();
                let statement_result = statement.trim();
                if statement_result.starts_with('?') {
                    //println!("Got {}", statement_result.unwrap());
                    repl_commands(statement_result, state, args);
                    state.statement_buf.clear();
                } else if state.bracket_depth == 0
                    && state.in_literal.is_none()
                    && state.curly_depth == 0
                {
                    state.statement_buf.push(';');
                    let complete_statement = state.statement_buf.iter().collect::<String>();
                    state
                        .writer
                        .write(complete_statement.as_bytes())
                        .expect("Failed to write to MPS interpreter");
                    execute(state, args);
                    state.statement_buf.clear();
                }
                state.statement_buf.push('\n');
                state.cursor_rightward_position = 0;
                // history
                let last_line = state.current_line.iter().collect::<String>();
                state.current_line.clear();
                if !last_line.is_empty()
                    && ((!state.history.is_empty()
                        && state.history[state.history.len() - 1] != last_line)
                        || state.history.is_empty())
                {
                    state.history.push(last_line);
                }
                state.selected_history = 0;

                prompt(state, args);
            }
            Key::ArrowUp => {
                if state.selected_history != state.history.len() {
                    state.selected_history += 1;
                    display_history_line(state, args);
                }
            }
            Key::ArrowDown => {
                if state.selected_history > 1 {
                    state.selected_history -= 1;
                    display_history_line(state, args);
                } else if state.selected_history == 1 {
                    state.selected_history = 0;
                    state.line_number -= 1;
                    state
                        .terminal
                        .clear_line()
                        .expect(TERMINAL_WRITE_ERROR);
                    prompt(state, args);
                    // clear stale input buffer
                    state.statement_buf.clear();
                    state.current_line.clear();
                    state.in_literal = None;
                    state.bracket_depth = 0;
                    state.curly_depth = 0;
                }
            }
            Key::ArrowLeft => {
                if state.current_line.len() > state.cursor_rightward_position {
                    state
                        .terminal
                        .move_cursor_left(1)
                        .expect(TERMINAL_WRITE_ERROR);
                    state.cursor_rightward_position += 1;
                }
            }
            Key::ArrowRight => {
                if state.cursor_rightward_position != 0 {
                    state
                        .terminal
                        .move_cursor_right(1)
                        .expect(TERMINAL_WRITE_ERROR);
                    state.cursor_rightward_position -= 1;
                }
            }
            _ => continue,
        }

        //println!("Read {}", read_buf[0]);
    }
}

#[inline(always)]
fn prompt(state: &mut ReplState, args: &CliArgs) {
    write!(state.terminal, "{}{}", state.line_number, args.prompt)
        .expect(TERMINAL_WRITE_ERROR);
    state.line_number += 1;
    state
        .terminal
        .flush()
        .expect("Failed to flush terminal output");
}

#[inline(always)]
fn display_history_line(state: &mut ReplState, args: &CliArgs) {
    // get historical line
    state.line_number -= 1;
    state
        .terminal
        .clear_line()
        .expect(TERMINAL_WRITE_ERROR);
    prompt(state, args);
    let new_statement = state.history[state.history.len() - state.selected_history].trim();
    state
        .terminal
        .write(new_statement.as_bytes())
        .expect(TERMINAL_WRITE_ERROR);
    // clear stale input buffer
    state.statement_buf.clear();
    state.current_line.clear();
    state.in_literal = None;
    state.bracket_depth = 0;
    state.curly_depth = 0;
    state.statement_buf.extend(new_statement.chars());
    state.current_line.extend(new_statement.chars());
}

#[inline(always)]
fn error_prompt(error: muss_player::PlayerError, args: &CliArgs) {
    eprintln!("E{}{}", args.prompt, error);
}

fn repl_commands(command_str: &str, state: &mut ReplState, _args: &CliArgs) {
    let words: Vec<&str> = command_str.split(' ').map(|s| s.trim()).collect();
    match words[0] {
        "?help" => writeln!(state.terminal, "{}", super::help::HELP_STRING).expect(TERMINAL_WRITE_ERROR),
        "?function" | "?functions" => writeln!(state.terminal, "{}", super::help::FUNCTIONS).expect(TERMINAL_WRITE_ERROR),
        "?filter" | "?filters" => writeln!(state.terminal, "{}", super::help::FILTERS).expect(TERMINAL_WRITE_ERROR),
        "?sort" | "?sorter" | "?sorters" => writeln!(state.terminal, "{}", super::help::SORTERS).expect(TERMINAL_WRITE_ERROR),
        "?proc" | "?procedure" | "?procedures" => writeln!(state.terminal, "{}", super::help::PROCEDURES).expect(TERMINAL_WRITE_ERROR),
        "?list" => {
            {
                let mut debug_state = DEBUG_STATE.write().expect("Failed to get write lock for debug state");
                debug_state.debug_flag = DebugFlag::List;
            }
            writeln!(state.terminal, "Listing upcoming items").expect(TERMINAL_WRITE_ERROR);

        },
        "?skip" => {
            {
                let mut debug_state = DEBUG_STATE.write().expect("Failed to get write lock for debug state");
                debug_state.debug_flag = DebugFlag::Skip;
            }
            writeln!(state.terminal, "Skipping upcoming items").expect(TERMINAL_WRITE_ERROR);
        },
        "?normal" => {
            {
                let mut debug_state = DEBUG_STATE.write().expect("Failed to get write lock for debug state");
                debug_state.debug_flag = DebugFlag::Normal;
            }
            writeln!(state.terminal, "Resuming normal operation").expect(TERMINAL_WRITE_ERROR);
        },
        "?verbose" => {
            let verbose = {
                let mut debug_state = DEBUG_STATE.write().expect("Failed to get write lock for debug state");
                debug_state.verbose = !debug_state.verbose;
                debug_state.verbose
            };
            writeln!(state.terminal, "Verbosed toggled to {}", verbose).expect(TERMINAL_WRITE_ERROR);
        },
        "?commands" => writeln!(state.terminal, "{}", super::help::REPL_COMMANDS).expect(TERMINAL_WRITE_ERROR),
        _ => writeln!(state.terminal, "Unknown command, try ?help").expect(TERMINAL_WRITE_ERROR),
    }
}
