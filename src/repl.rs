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
}

impl ReplState {
    fn new(chan_writer: ChannelWriter) -> Self {
        Self {
            stdin: io::stdin(),
            line_number: 0,
            statement_buf: Vec::new(),
            writer: chan_writer,
        }
    }
}

pub fn repl(args: CliArgs) {
    let (writer, reader) = channel_io();
    let player_builder = move || {
        let runner = MpsRunner::with_stream(reader);
        let player = MpsPlayer::new(runner).unwrap();
        player
    };
    let mut state = ReplState::new(writer);
    if let Some(playlist_file) = &args.playlist {
        println!("Playlist mode (output: `{}`)", playlist_file);
        let mut player = player_builder();
        let mut playlist_writer = io::BufWriter::new(std::fs::File::create(playlist_file).expect(
            &format!("Abort: Cannot create writeable file `{}`", playlist_file),
        ));
        read_loop(&args, &mut state, || {
            match player.save_m3u8(&mut playlist_writer) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e.message()),
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
                    Err(e) => eprintln!("{}", e.message()),
                }
            } else {
                for e in ctrl.check_ack() {
                    eprintln!("{}", e.message());
                }
            }
        });
    }
}

fn read_loop<F: FnMut()>(args: &CliArgs, state: &mut ReplState, mut execute: F) -> ! {
    let mut read_buf: [u8; 1] = [0];
    prompt(&mut state.line_number, args);
    loop {
        read_buf[0] = 0;
        while read_buf[0] == 0 {
            // TODO: enable raw mode (char by char) reading of stdin
            state
                .stdin
                .read(&mut read_buf)
                .expect("Failed to read stdin");
        }
        match read_buf[0] as char {
            '\n' => {
                state.statement_buf.push(read_buf[0]);
                let statement_result = std::str::from_utf8(state.statement_buf.as_slice());
                if statement_result.is_ok() && statement_result.unwrap().starts_with("?") {
                    repl_commands(statement_result.unwrap());
                    state.writer.write(&[';' as u8]).unwrap_or(0);
                } else {
                    state
                        .writer
                        .write(state.statement_buf.as_slice())
                        .expect("Failed to write to MPS interpreter");
                    execute();
                }
                state.statement_buf.clear();
                prompt(&mut state.line_number, args);
            }
            _ => state.statement_buf.push(read_buf[0]),
        }
    }
}

#[inline(always)]
fn prompt(line: &mut usize, args: &CliArgs) {
    print!("{}{}", line, args.prompt);
    *line += 1;
    std::io::stdout().flush().expect("Failed to flush stdout");
}

fn repl_commands(command_str: &str) {
    let words: Vec<&str> = command_str.split(" ").map(|s| s.trim()).collect();
    match words[0] {
        "?help" => println!("{}", super::help::help()),
        "?function" | "?functions" => println!("{}", super::help::functions()),
        "?filter" | "?filters" => println!("{}", super::help::filters()),
        _ => println!("Unknown command, try ?help"),
    }
}
