//! A Muss playback library with support for media controls (Linux & D-Bus only atm).
//! This handles the output from interpreting a script.
//! Music playback and m3u8 playlist generation are implemented in this part of the project.
//!

#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::redundant_field_names)]

mod controller;
mod errors;
pub(crate) mod os_controls;
mod player;
pub(crate) mod player_wrapper;
pub(crate) mod uri;
//mod utility;

pub use controller::Controller;
pub use errors::{PlaybackError, PlayerError, UriError};
#[cfg(feature = "mpd")]
pub use player::mpd_connection;
pub use player::Player;
//pub use utility::{play_script};

#[cfg(test)]
mod tests {}
