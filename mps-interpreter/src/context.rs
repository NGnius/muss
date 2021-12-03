use std::fmt::{Debug, Display, Formatter, Error};

#[derive(Debug)]
pub struct MpsContext {
    pub sqlite_connection: Option<rusqlite::Connection>,
}

impl Default for MpsContext {
    fn default() -> Self {
        Self {
            sqlite_connection: None, // initialized by first SQL statement instead
        }
    }
}

impl std::clone::Clone for MpsContext {
    fn clone(&self) -> Self {
        Self {
            sqlite_connection: None,
        }
    }
}

impl Display for MpsContext {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "MpsContext")?;
        Ok(())
    }
}
