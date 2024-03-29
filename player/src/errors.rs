use std::convert::From;
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug, Clone)]
pub enum PlayerError {
    Playback(PlaybackError),
    Uri(UriError),
    #[cfg(feature = "mpd")]
    Mpd(String),
}

impl PlayerError {
    pub(crate) fn from_err_playback<E: Display>(err: E) -> Self {
        Self::Playback(PlaybackError::from_err(err))
    }

    pub(crate) fn from_file_err_playback<E: Display, P: AsRef<std::path::Path>>(err: E, path: P) -> Self {
        Self::Playback(PlaybackError { msg: format!("{}: `{}`", err, path.as_ref().display()) })
    }

    /*pub(crate) fn from_err_uri<E: Display>(err: E) -> Self {
        Self::Uri(UriError::from_err(err))
    }*/

    #[cfg(feature = "mpd")]
    pub(crate) fn from_err_mpd<E: Display>(err: E) -> Self {
        Self::Mpd(format!("{}", err))
    }
}

impl Display for PlayerError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Playback(p) => (p as &dyn Display).fmt(f),
            Self::Uri(u) => (u as &dyn Display).fmt(f),
            #[cfg(feature = "mpd")]
            Self::Mpd(m) => (m as &dyn Display).fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaybackError {
    pub(crate) msg: String,
}

impl PlaybackError {
    pub fn from_err<E: Display>(err: E) -> Self {
        Self {
            msg: format!("{}", err),
        }
    }

    pub fn message(&self) -> &'_ str {
        &self.msg
    }
}

impl Display for PlaybackError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "PlaybackError: {}", &self.msg)
    }
}

impl From<PlaybackError> for PlayerError {
    fn from(other: PlaybackError) -> Self {
        PlayerError::Playback(other)
    }
}

#[derive(Debug, Clone)]
pub enum UriError {
    Unsupported(String),
    Message(String),
}

impl UriError {
    pub fn from_err<E: Display>(err: E) -> Self {
        Self::Message(format!("{}", err))
    }
}

impl Display for UriError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "UriError: ")?;
        match self {
            Self::Unsupported(scheme) => write!(f, "Unsupported URI `{}//`", scheme),
            Self::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<UriError> for PlayerError {
    fn from(other: UriError) -> Self {
        PlayerError::Uri(other)
    }
}
