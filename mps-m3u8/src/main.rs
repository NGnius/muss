//! Minimal CLI tool to generate a m3u8 playlist from a MPS file.
//! This does not support playback, so it can run on any platform with a filesystem.
//! Use `mps-m3u8 --help` for usage instructions.
//!

mod cli;

use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::path::Path;

use m3u8_rs::{MediaPlaylist, MediaSegment};

use mps_interpreter::{MpsFaye, MpsItem};

fn main() {
    let args = cli::parse();

    let out_path = Path::new(&args.playlist);
    let mut out_file = BufWriter::new(File::create(out_path).expect("Invalid output file"));

    let mut playlist = MediaPlaylist {
        version: 6,
        ..Default::default()
    };
    let mut skipped_count = 0;

    if args.raw {
        println!("Executing: {}", &args.input);
        let in_file = Cursor::new(&args.input);

        let runner = MpsFaye::with_stream(in_file);
        for item in runner {
            match item {
                Ok(music) => {
                    if let Some(filename) =
                        music_filename(&music)
                    {
                        playlist.segments.push(MediaSegment {
                            uri: filename,
                            title: music_title(&music),
                            ..Default::default()
                        });
                    } else {
                        skipped_count += 1;
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    } else {
        let in_path = Path::new(&args.input);
        let in_file = BufReader::new(File::open(in_path).expect("Invalid/missing input file"));

        let runner = MpsFaye::with_stream(in_file);
        for item in runner {
            match item {
                Ok(music) => {
                    if let Some(filename) =
                        music_filename(&music)
                    {
                        playlist.segments.push(MediaSegment {
                            uri: filename,
                            title: music_title(&music),
                            ..Default::default()
                        });
                    } else {
                        skipped_count += 1;
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    }
    if skipped_count != 0 {
        eprintln!(
            "Skipped {} items due to missing `filename` field",
            skipped_count
        );
    }
    if let Err(e) = playlist.write_to(&mut out_file) {
        eprintln!("Playlist save error: {}", e);
    }
}

fn music_title(item: &MpsItem) -> Option<String> {
    item.field("title").and_then(|x| x.to_owned().to_str())
}

fn music_filename(item: &MpsItem) -> Option<String> {
    if let Some(filename) = item.field("filename") {
        if let Ok(cwd) = std::env::current_dir() {
            let path: &Path = &cwd;
            Some(filename.as_str().replace(path.to_str().unwrap_or(""), "./"))
        } else {
            Some(filename.to_string())
        }
    } else {
        None
    }
}


