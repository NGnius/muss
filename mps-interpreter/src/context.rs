use super::processing::database::{MpsDatabaseQuerier, MpsSQLiteExecutor};
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug)]
pub struct MpsContext {
    pub database: Box<dyn MpsDatabaseQuerier>,
}

impl Default for MpsContext {
    fn default() -> Self {
        Self {
            database: Box::new(MpsSQLiteExecutor::default()),
        }
    }
}

impl std::clone::Clone for MpsContext {
    fn clone(&self) -> Self {
        Self {
            database: Box::new(MpsSQLiteExecutor::default()),
        }
    }
}

impl Display for MpsContext {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "MpsContext")?;
        Ok(())
    }
}
