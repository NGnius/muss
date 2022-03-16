//! Read, Execute, Print Loop functionality

use std::io::{self, Write};

use console::{Term, Key};

use mps_interpreter::MpsRunner;
use mps_player::{MpsController, MpsPlayer};

use super::channel_io::{channel_io, ChannelWriter};
use super::cli::CliArgs;

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
}

impl ReplState {
    fn new(chan_writer: ChannelWriter, term: Term) -> Self {
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
        }
    }
}

pub fn repl(args: CliArgs) {
    let term = Term::stdout();
    term.set_title("mps");
    let (writer, reader) = channel_io();
    let volume = args.volume.clone();
    let player_builder = move || {
        let runner = MpsRunner::with_stream(reader);

        let player = MpsPlayer::new(runner).unwrap();
        if let Some(vol) = volume {
            player.set_volume(vol);
        }
        player
    };
    let mut state = ReplState::new(writer, term);
    if let Some(playlist_file) = &args.playlist {
        writeln!(state.terminal, "Playlist mode (output: `{}`)", playlist_file).expect("Failed to write to terminal output");
        let mut player = player_builder();
        let mut playlist_writer =
            io::BufWriter::new(std::fs::File::create(playlist_file).unwrap_or_else(|_| {
                panic!("Abort: Cannot create writeable file `{}`", playlist_file)
            }));
        read_loop(&args, &mut state, || {
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
        });
    } else {
        writeln!(state.terminal, "Playback mode (output: audio device)").expect("Failed to write to terminal output");
        let ctrl = MpsController::create_repl(player_builder);
        read_loop(&args, &mut state, || {
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
        });
    }
}

fn read_loop<F: FnMut()>(args: &CliArgs, state: &mut ReplState, mut execute: F) -> ! {
    prompt(state, args);
    loop {
        match state.terminal.read_key().expect("Failed to read terminal input") {
            Key::Char(read_c) => {
                if state.cursor_rightward_position == 0 {
                    write!(state.terminal, "{}", read_c).expect("Failed to write to terminal output");
                    state.statement_buf.push(read_c);
                    state.current_line.push(read_c);
                } else {
                    write!(state.terminal, "{}", read_c).expect("Failed to write to terminal output");
                    for i in state.current_line.len() - state.cursor_rightward_position .. state.current_line.len() {
                        write!(state.terminal, "{}", state.current_line[i]).expect("Failed to write to terminal output");
                    }
                    state.terminal.move_cursor_left(state.cursor_rightward_position).expect("Failed to write to terminal output");
                    state.statement_buf.insert(state.statement_buf.len() - state.cursor_rightward_position, read_c);
                    state.current_line.insert(state.current_line.len() - state.cursor_rightward_position, read_c);
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
                    ')' => if state.bracket_depth != 0 { state.bracket_depth -= 1 },
                    '{' => state.curly_depth += 1,
                    '}' => if state.curly_depth != 0 { state.curly_depth -= 1 },
                    ';' => {
                        if state.in_literal.is_none() {
                            state
                                .writer
                                .write(state.statement_buf.iter().collect::<String>().as_bytes())
                                .expect("Failed to write to MPS interpreter");
                            execute();
                            state.statement_buf.clear();
                        }
                    }
                    '\n' => {
                        let statement = state.statement_buf.iter().collect::<String>();
                        let statement_result = statement.trim();
                        if statement_result.starts_with('?') {
                            //println!("Got {}", statement_result.unwrap());
                            repl_commands(statement_result);
                            state.statement_buf.clear();
                        } else if state.bracket_depth == 0 && state.in_literal.is_none() && state.curly_depth == 0 {
                            state.statement_buf.push(';');
                            state
                                .writer
                                .write(state.statement_buf.iter().collect::<String>().as_bytes())
                                .expect("Failed to write to MPS interpreter");
                            execute();
                            state.statement_buf.clear();
                        }
                        prompt(state, args);
                    }
                    _ => {}
                }
            },
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
                            },
                            '(' => if state.bracket_depth != 0 { state.bracket_depth -= 1 },
                            ')' => state.bracket_depth += 1,
                            '{' => if state.curly_depth != 0 { state.curly_depth -= 1 },
                            '}' => state.curly_depth += 1,
                            _ => {},
                        }
                        match c {
                            '\n' | '\r' => {
                                // another line, cannot backspace that far
                                state.statement_buf.push(c);
                            },
                            _ => {
                                state.current_line.pop();
                                state.terminal.move_cursor_left(1).expect("Failed to write to terminal output");
                                write!(state.terminal, " ").expect("Failed to write to terminal output");
                                state.terminal.flush().expect("Failed to flush terminal output");
                                state.terminal.move_cursor_left(1).expect("Failed to write to terminal output");
                            }
                        }
                    }
                } else {
                    if state.current_line.len() != state.cursor_rightward_position {
                        // if not at start of line
                        let removed_char = state.current_line.remove(state.current_line.len()-state.cursor_rightward_position-1);
                        state.statement_buf.remove(state.statement_buf.len()-state.cursor_rightward_position-1);
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
                            },
                            '(' => if state.bracket_depth != 0 { state.bracket_depth -= 1 },
                            ')' => state.bracket_depth += 1,
                            '{' => if state.curly_depth != 0 { state.curly_depth -= 1 },
                            '}' => state.curly_depth += 1,
                            _ => {},
                        }
                        // re-print end of line to remove character in middle
                        state.terminal.move_cursor_left(1).expect("Failed to write to terminal output");
                        for i in state.current_line.len() - state.cursor_rightward_position .. state.current_line.len() {
                            write!(state.terminal, "{}", state.current_line[i]).expect("Failed to write to terminal output");
                        }
                        write!(state.terminal, " ").expect("Failed to write to terminal output");
                        state.terminal.move_cursor_left(state.cursor_rightward_position + 1).expect("Failed to write to terminal output");
                    }
                }

            },
            Key::Enter => {
                state.terminal.write_line("").expect("Failed to write to terminal output");
                let statement = state.statement_buf.iter().collect::<String>();
                let statement_result = statement.trim();
                if statement_result.starts_with('?') {
                    //println!("Got {}", statement_result.unwrap());
                    repl_commands(statement_result);
                    state.statement_buf.clear();
                } else if state.bracket_depth == 0 && state.in_literal.is_none() && state.curly_depth == 0 {
                    state.statement_buf.push(';');
                    let complete_statement = state.statement_buf.iter().collect::<String>();
                    state
                        .writer
                        .write(complete_statement.as_bytes())
                        .expect("Failed to write to MPS interpreter");
                    execute();
                    state.statement_buf.clear();
                }
                state.statement_buf.push('\n');
                state.cursor_rightward_position = 0;
                // history
                let last_line = state.current_line.iter().collect::<String>();
                state.current_line.clear();
                if !last_line.is_empty() && ((!state.history.is_empty() && state.history[state.history.len()-1] != last_line) || state.history.is_empty()) {
                    state.history.push(last_line);
                }
                state.selected_history = 0;

                prompt(state, args);
            },
            Key::ArrowUp => {
                if state.selected_history != state.history.len() {
                    state.selected_history += 1;
                    display_history_line(state, args);
                }
            },
            Key::ArrowDown => {
                if state.selected_history > 1 {
                    state.selected_history -= 1;
                    display_history_line(state, args);
                } else if state.selected_history == 1 {
                    state.selected_history = 0;
                    state.line_number -= 1;
                    state.terminal.clear_line().expect("Failed to write to terminal output");
                    prompt(state, args);
                    // clear stale input buffer
                    state.statement_buf.clear();
                    state.current_line.clear();
                    state.in_literal = None;
                    state.bracket_depth = 0;
                    state.curly_depth = 0;
                }
            },
            Key::ArrowLeft => {
                if state.current_line.len() > state.cursor_rightward_position {
                    state.terminal.move_cursor_left(1).expect("Failed to write to terminal output");
                    state.cursor_rightward_position += 1;
                }
            },
            Key::ArrowRight => {
                if state.cursor_rightward_position != 0 {
                    state.terminal.move_cursor_right(1).expect("Failed to write to terminal output");
                    state.cursor_rightward_position -= 1;
                }
            },
            _ => continue
        }
        
        //println!("Read {}", read_buf[0]);
        
    }
}

#[inline(always)]
fn prompt(state: &mut ReplState, args: &CliArgs) {
    write!(state.terminal, "{}{}", state.line_number, args.prompt).expect("Failed to write to terminal output");
    state.line_number += 1;
    state.terminal.flush().expect("Failed to flush terminal output");
}

#[inline(always)]
fn display_history_line(state: &mut ReplState, args: &CliArgs) {
    // get historical line
    state.line_number -= 1;
    state.terminal.clear_line().expect("Failed to write to terminal output");
    prompt(state, args);
    let new_statement = state.history[state.history.len() - state.selected_history].trim();
    state.terminal.write(new_statement.as_bytes()).expect("Failed to write to terminal output");
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
fn error_prompt(error: mps_player::PlaybackError, args: &CliArgs) {
    eprintln!("E{}{}", args.prompt, error.message());
}

fn repl_commands(command_str: &str) {
    let words: Vec<&str> = command_str.split(' ').map(|s| s.trim()).collect();
    match words[0] {
        "?help" => println!("{}", super::help::HELP_STRING),
        "?function" | "?functions" => println!("{}", super::help::FUNCTIONS),
        "?filter" | "?filters" => println!("{}", super::help::FILTERS),
        "?sort" | "?sorter" | "?sorters" => println!("{}", super::help::SORTERS),
        "?proc" | "?procedure" | "?procedures" => println!("{}", super::help::PROCEDURES),
        _ => println!("Unknown command, try ?help"),
    }
}
