//! An MPS playback library with support for Linux media controls (D-Bus).
//!

mod controller;
mod errors;
pub(crate) mod os_controls;
mod player;
pub(crate) mod player_wrapper;
//mod utility;

pub use controller::MpsController;
pub use errors::PlaybackError;
pub use player::MpsPlayer;
//pub use utility::{play_script};

#[cfg(test)]
mod tests {}
