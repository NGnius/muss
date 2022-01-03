//! An MPS program which plays music.
//! This doesn't do much yet, since mps-interpreter is still under construction.
//!
//! Future home of a MPS REPL for playing music ergonomically through a CLI.
//!

mod channel_io;
mod cli;
mod repl;

use std::io;
use std::path::PathBuf;

use mps_interpreter::MpsRunner;
use mps_player::{MpsPlayer, PlaybackError, MpsController};

#[allow(dead_code)]
fn play_cursor() -> Result<(), PlaybackError> {
    let cursor = io::Cursor::<&'static str>::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
    let runner = MpsRunner::with_stream(cursor);
    let mut player = MpsPlayer::new(runner)?;
    player.play_all()
}

fn main() {
    let args = cli::parse();

    if let Some(script_file) = &args.file {
        // interpret script
        // script file checks
        if let Err(_) = file_checks(script_file) {
            return;
        }
        // build playback controller
        let script_file2 = script_file.clone();
        let player_builder = move || {
            let script_reader = io::BufReader::new(
                std::fs::File::open(&script_file2)
                    .expect(&format!("Abort: Cannot open file `{}`", &script_file2))
            );
            let runner = MpsRunner::with_stream(script_reader);
            let player = MpsPlayer::new(runner).unwrap();
            player
        };
        if let Some(playlist_file) = &args.playlist {
            // generate playlist
            let mut player = player_builder();
            let mut writer = io::BufWriter::new(
                std::fs::File::create(playlist_file)
                    .expect(&format!("Abort: Cannot create writeable file `{}`", playlist_file))
            );
            match player.save_m3u8(&mut writer) {
                Ok(_) => println!("Succes: Finished playlist `{}` from script `{}`", playlist_file, script_file),
                Err(e) => eprintln!("{}", e),
            }
        } else {
            // live playback
            let ctrl = MpsController::create(player_builder);
            match ctrl.wait_for_done() {
                Ok(_) => println!("Succes: Finished playback from script `{}`", script_file),
                Err(e) => eprintln!("{}", e),
            }
        }
    } else {
        // start REPL
        println!("Welcome to MPS interactive mode!");
        println!("End a statement with ; to execute it.");
        repl::repl(args)
    }
}

fn file_checks(path_str: &str) -> Result<(), ()> {
    let path = PathBuf::from(path_str);
    if !path.exists() {
        eprintln!("Abort: File `{}` does not exist", path_str);
        return Err(());
    }
    if !path.is_file() {
        eprintln!("Abort: Path `{}` is not a file", path_str);
        return Err(());
    }
    Ok(())
}
