//! A Muss playback library with support for media controls (Linux & D-Bus only atm).
//! This handles the output from interpreting a script.
//! Music playback and m3u8 playlist generation are implemented in this part of the project.
//!

mod controller;
mod errors;
pub(crate) mod os_controls;
mod player;
pub(crate) mod player_wrapper;
pub(crate) mod uri;
//mod utility;

pub use controller::Controller;
pub use errors::{PlaybackError, UriError, PlayerError};
pub use player::Player;
#[cfg(feature = "mpd")]
pub use player::mpd_connection;
//pub use utility::{play_script};

#[cfg(test)]
mod tests {}
