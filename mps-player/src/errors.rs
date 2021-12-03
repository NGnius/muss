use std::fmt::{Debug, Display, Formatter, Error};

#[derive(Debug)]
pub struct PlaybackError {
    pub(crate) msg: String
}

impl PlaybackError {
    pub fn from_err<E: Display>(err: E) -> Self {
        Self {
            msg: format!("{}", err),
        }
    }
}

impl Display for PlaybackError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "PlaybackError: {}", &self.msg)
    }
}
