use super::processing::database::{MpsDatabaseQuerier, MpsSQLiteExecutor};
use super::processing::general::{MpsVariableStorer, MpsOpStorage};
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug)]
pub struct MpsContext {
    pub database: Box<dyn MpsDatabaseQuerier>,
    pub variables: Box<dyn MpsVariableStorer>,
}

impl Default for MpsContext {
    fn default() -> Self {
        Self {
            database: Box::new(MpsSQLiteExecutor::default()),
            variables: Box::new(MpsOpStorage::default()),
        }
    }
}

impl Display for MpsContext {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "MpsContext")?;
        Ok(())
    }
}
