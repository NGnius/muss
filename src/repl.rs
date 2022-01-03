//! Read, Execute, Print Loop functionality

use std::io::{self, Write, Read};

use mps_interpreter::MpsRunner;
use mps_player::{MpsPlayer, MpsController};

use super::cli::CliArgs;
use super::channel_io::channel_io;

pub fn repl(args: CliArgs) {
    let (mut writer, reader) = channel_io();
    let player_builder = move || {
        let runner = MpsRunner::with_stream(reader);
        let player = MpsPlayer::new(runner).unwrap();
        player
    };
    let stdin_t = io::stdin();
    let mut stdin = stdin_t.lock();
    let mut buf: Vec<u8> = Vec::new();
    let mut read_buf = [0];
    let mut current_line = 0;
    // TODO: enable raw mode (char by char) reading of stdin
    // TODO: generalize loop for better code reuse between playlist and playback mode
    if let Some(playlist_file) = &args.playlist {
        println!("Playlist mode (output: `{}`)", playlist_file);
        let mut player = player_builder();
        let mut playlist_writer = io::BufWriter::new(
            std::fs::File::create(playlist_file)
                .expect(&format!("Abort: Cannot create writeable file `{}`", playlist_file))
        );
        prompt(&mut current_line);
        loop {
            read_buf[0] = 0;
            while read_buf[0] == 0 {
                stdin.read_exact(&mut read_buf).expect("Failed to read stdin");
            }
            match read_buf[0] as char {
                ';' => {
                    buf.push(read_buf[0]);
                    writer.write(buf.as_slice()).expect("Failed to write to MPS interpreter");
                    match player.save_m3u8(&mut playlist_writer) {
                        Ok(_) => {},
                        Err(e) => eprintln!("{}", e.message()),
                    }
                    buf.clear();
                    playlist_writer.flush().expect("Failed to flush playlist to file");
                    prompt(&mut current_line);
                },
                /*
                '\x27' => break, // ESC key
                '\x03' => break, // Ctrl+C
                */
                _ => buf.push(read_buf[0]),
            }
        }
    } else {
        println!("Playback mode (output: audio device)");
        let ctrl = MpsController::create_repl(player_builder);
        prompt(&mut current_line);
        loop {
            read_buf[0] = 0;
            while read_buf[0] == 0 {
                stdin.read_exact(&mut read_buf).expect("Failed to read stdin");
            }
            match read_buf[0] as char {
                ';' => {
                    buf.push(read_buf[0]);
                    writer.write(buf.as_slice()).expect("Failed to write to MPS interpreter");
                    //ctrl.play().expect("Failed to start playback");
                    if args.wait {
                        match ctrl.wait_for_empty() {
                            Ok(_) => {},
                            Err(e) => eprintln!("{}", e.message()),
                        }
                    } else {
                        for e in ctrl.check_ack() {
                            eprintln!("{}", e.message());
                        }
                    }
                    buf.clear();
                    prompt(&mut current_line);
                },
                '\x27' => break, // ESC key
                '\x03' => break, // Ctrl+C
                _ => buf.push(read_buf[0]),
            }
        }
    }
}

#[inline(always)]
fn prompt(line: &mut usize) {
    print!("{} |-> ", line);
    *line += 1;
    std::io::stdout().flush().expect("Failed to flush stdout");
}
