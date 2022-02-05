use super::processing::database::{MpsDatabaseQuerier, MpsSQLiteExecutor};
use super::processing::general::{
    MpsFilesystemExecutor, MpsFilesystemQuerier, MpsOpStorage, MpsVariableStorer,
};
#[cfg(feature = "advanced")]
use super::processing::advanced::{
    MpsMusicAnalyzer, MpsDefaultAnalyzer
};
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug)]
pub struct MpsContext {
    pub database: Box<dyn MpsDatabaseQuerier>,
    pub variables: Box<dyn MpsVariableStorer>,
    pub filesystem: Box<dyn MpsFilesystemQuerier>,
    #[cfg(feature = "advanced")]
    pub analysis: Box<dyn MpsMusicAnalyzer>,
}

impl Default for MpsContext {
    fn default() -> Self {
        Self {
            database: Box::new(MpsSQLiteExecutor::default()),
            variables: Box::new(MpsOpStorage::default()),
            filesystem: Box::new(MpsFilesystemExecutor::default()),
            #[cfg(feature = "advanced")]
            analysis: Box::new(MpsDefaultAnalyzer::default()),
        }
    }
}

impl Display for MpsContext {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "MpsContext{{...}}")?;
        Ok(())
    }
}
