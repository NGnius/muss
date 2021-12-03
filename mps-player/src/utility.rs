use std::path::Path;
use std::io;
use std::fs;

use mps_interpreter::{MpsRunner, tokens::MpsTokenReader};

use super::{MpsPlayer, PlaybackError};

pub fn play_script<P: AsRef<Path>>(p: P) -> Result<(), PlaybackError> {
    let file = fs::File::open(music.filename).map_err(PlaybackError::from_err)?;
    let stream = io::BufReader::new(file);
    let runner = MpsRunner::with_stream(stream);
    let mut player = MpsPlayer::new(runner);
    player.play_all()
}
