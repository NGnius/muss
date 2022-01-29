//! Read, Execute, Print Loop functionality

use std::io::{self, Read, Stdin, Write};

use mps_interpreter::MpsRunner;
use mps_player::{MpsController, MpsPlayer};

use super::channel_io::{channel_io, ChannelWriter};
use super::cli::CliArgs;

struct ReplState {
    stdin: Stdin,
    line_number: usize,
    statement_buf: Vec<u8>,
    writer: ChannelWriter,
    in_literal: Option<char>,
    bracket_depth: usize,
}

impl ReplState {
    fn new(chan_writer: ChannelWriter) -> Self {
        Self {
            stdin: io::stdin(),
            line_number: 0,
            statement_buf: Vec::new(),
            writer: chan_writer,
            in_literal: None,
            bracket_depth: 0,
        }
    }
}

pub fn repl(args: CliArgs) {
    /*let mut terminal = termios::Termios::from_fd(0 /* stdin */).unwrap();
    terminal.c_lflag &= !termios::ICANON; // no echo and canonical mode
    termios::tcsetattr(0, termios::TCSANOW, &mut terminal).unwrap();*/
    let (writer, reader) = channel_io();
    let player_builder = move || {
        let runner = MpsRunner::with_stream(reader);
        
        MpsPlayer::new(runner).unwrap()
    };
    let mut state = ReplState::new(writer);
    if let Some(playlist_file) = &args.playlist {
        println!("Playlist mode (output: `{}`)", playlist_file);
        let mut player = player_builder();
        let mut playlist_writer = io::BufWriter::new(std::fs::File::create(playlist_file).unwrap_or_else(|_| panic!("Abort: Cannot create writeable file `{}`", playlist_file)));
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
        println!("Playback mode (output: audio device)");
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
    let mut read_buf: [u8; 1] = [0];
    prompt(&mut state.line_number, args);
    loop {
        let mut read_count = 0;
        //read_buf[0] = 0;
        while read_count == 0 {
            // TODO: enable raw mode (char by char) reading of stdin
            read_count = state
                .stdin
                .read(&mut read_buf)
                .expect("Failed to read stdin");
        }
        //println!("Read {}", read_buf[0]);
        state.statement_buf.push(read_buf[0]);
        match read_buf[0] as char {
            '"' | '`' => {
                if let Some(c) = state.in_literal {
                    if c == read_buf[0] as char {
                        state.in_literal = None;
                    }
                } else {
                    state.in_literal = Some(read_buf[0] as char);
                }
            }
            '(' => state.bracket_depth += 1,
            ')' => state.bracket_depth -= 1,
            ';' => {
                if state.in_literal.is_none() {
                    state
                        .writer
                        .write(state.statement_buf.as_slice())
                        .expect("Failed to write to MPS interpreter");
                    execute();
                    state.statement_buf.clear();
                }
            }
            '\n' => {
                let statement_result = std::str::from_utf8(state.statement_buf.as_slice());
                if statement_result.is_ok() && statement_result.unwrap().trim().starts_with('?') {
                    //println!("Got {}", statement_result.unwrap());
                    repl_commands(statement_result.unwrap().trim());
                    state.statement_buf.clear();
                } else if state.bracket_depth == 0 && state.in_literal.is_none() {
                    state.statement_buf.push(b';');
                    state
                        .writer
                        .write(state.statement_buf.as_slice())
                        .expect("Failed to write to MPS interpreter");
                    execute();
                    state.statement_buf.clear();
                }
                prompt(&mut state.line_number, args);
            }
            _ => {}
        }
    }
}

#[inline(always)]
fn prompt(line: &mut usize, args: &CliArgs) {
    print!("{}{}", line, args.prompt);
    *line += 1;
    std::io::stdout().flush().expect("Failed to flush stdout");
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
        _ => println!("Unknown command, try ?help"),
    }
}
