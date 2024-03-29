//! Sort, filter and analyse your music to create great playlists.
//! This project implements the interpreter (`./interpreter`), music player (`./player`), and CLI interface for Muss (`./`).
//! The CLI interface includes a REPL for running scripts.
//! The REPL interactive mode also provides more details about using Muss through the `?help` command.
//!
//! # Usage
//! To access the REPL, simply run `cargo run`. You will need the [Rust toolchain installed](https://rustup.rs/). For a bit of extra performance, run `cargo run --release` instead.
//!
//! # Examples
//!
//! ## One-liners
//!
//! All songs by artist `<artist>` (in your library), sorted by similarity to a random first song by the artist.
//! ```muss
//! files().(.artist? like "<artist>")~(~radio);
//! ```
//!
//! All songs with a `.flac` file extension (anywhere in their path -- not necessarily at the end).
//! ```muss
//! files().(.filename? like ".flac");
//! ```
//!
//! All songs by artist `<artist1>` or `<artist2>`, sorted by similarity to a random first song by either artist.
//! ```muss
//! files().(.artist? like "<artist1>" || .artist? like "<artist2>")~(~radio);
//! ```
//!
//! ## Bigger examples
//!
//! For now, check out `./src/tests`, `./player/tests`, and `./interpreter/tests` for examples.
//! One day I'll add pretty REPL example pictures and some script files...
//! // TODO
//!
//! # FAQ
//!
//! ## Can I use Muss right now?
//! **Sure!** It's never complete, but Muss is completely useable right now. Hopefully most of the bugs have been ironed out as well :)
//!
//! ## Why write a new language?
//! **I thought it would be fun**. I also wanted to be able to play my music without having to be at the whim of someone else's algorithm (and music), and playing just by album or artist was getting boring. Designing a language specifically for iteration seemed like a cool & novel way of doing it, too (though every approach is a novel approach for me).
//!
//! ## What is Muss?
//! **Music Set Script (MuSS) is a language for describing a playlist of music.** It can execute SQLite select clauses or directly query the filesystem. Queries can be modified by using filters, functions, and sorters built-in to Muss (see interpreter's README.md).
//!
//! ## Is Muss a scripting language?
//! **Yes**. It evolved from a simple query language into something that can do arbitrary calculations. Whether it's Turing-complete is still unproven, but it's powerful enough for what I want it to do.
//!

mod channel_io;
mod cli;
mod help;
mod repl;

use std::io;
use std::path::PathBuf;

use muss_interpreter::Interpreter;
use muss_player::{Controller, Player, PlayerError};

#[allow(dead_code)]
fn play_cursor() -> Result<(), PlayerError> {
    let cursor = io::Cursor::<&'static str>::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
    let runner = Interpreter::with_stream(cursor);
    let mut player = Player::new(runner)?;
    player.play_all()
}

fn main() {
    let args = cli::parse();
    if let Err(e) = cli::validate(&args) {
        eprintln!("{}", e);
        return;
    }

    if let Some(script_file) = &args.file {
        // interpret script
        // script file checks
        if file_checks(script_file).is_err() {
            return;
        }
        // build playback controller
        let script_file2 = script_file.clone();
        let volume = args.volume;
        let mpd = match args
            .mpd
            .clone()
            .map(|a| muss_player::mpd_connection(a.parse().unwrap()))
            .transpose()
        {
            Ok(mpd) => mpd,
            Err(e) => panic!("Abort: Cannot connect to MPD: {}", e),
        };
        let player_builder = move || {
            let script_reader = io::BufReader::new(
                std::fs::File::open(&script_file2)
                    .unwrap_or_else(|_| panic!("Abort: Cannot open file `{}`", &script_file2)),
            );
            let runner = Interpreter::with_stream(script_reader);

            let mut player = Player::new(runner).unwrap();
            if let Some(vol) = volume {
                player.set_volume(vol);
            }
            if let Some(mpd) = mpd {
                player.set_mpd(mpd);
            }
            player
        };
        if let Some(playlist_file) = &args.playlist {
            // generate playlist
            let mut player = player_builder();
            let mut writer =
                io::BufWriter::new(std::fs::File::create(playlist_file).unwrap_or_else(|_| {
                    panic!("Abort: Cannot create writeable file `{}`", playlist_file)
                }));
            match player.save_m3u8(&mut writer) {
                Ok(_) => println!(
                    "Success: Finished playlist `{}` from script `{}`",
                    playlist_file, script_file
                ),
                Err(e) => eprintln!("{}", e),
            }
        } else {
            // live playback
            let ctrl = Controller::create(player_builder);
            match ctrl.wait_for_done() {
                Ok(_) => println!("Success: Finished playback from script `{}`", script_file),
                Err(e) => eprintln!("{}", e),
            }
        }
    } else {
        // start REPL
        println!("Welcome to Muss interactive mode!");
        println!("Run ?help for usage instructions.");
        //println!("End a statement with ; to execute it.");
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
