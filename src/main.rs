use std::io;
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
    //play_cursor().unwrap();
    let ctrl = MpsController::create(|| {
        //let cursor = io::Cursor::<&'static str>::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
        let cursor = io::Cursor::<&'static str>::new(
        "sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);
        artist(`gwen`);"
        );
        let runner = MpsRunner::with_stream(cursor);
        let player = MpsPlayer::new(runner).unwrap();
        player.set_volume(0.8);
        player
    });

    ctrl.wait_for_done().unwrap();
    //ctrl.exit().unwrap(); // don't use both
}
