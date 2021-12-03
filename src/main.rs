use std::io;
use mps_interpreter::MpsRunner;
use mps_player::{MpsPlayer, PlaybackError};

#[allow(dead_code)]
fn play_cursor() -> Result<(), PlaybackError> {
    let cursor = io::Cursor::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
    let runner = MpsRunner::with_stream(cursor);
    let mut player = MpsPlayer::new(runner)?;
    player.play_all()
}

fn main() {
    play_cursor().unwrap();
}
