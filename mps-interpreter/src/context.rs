#[cfg(feature = "advanced")]
use super::processing::advanced::{MpsDefaultAnalyzer, MpsMusicAnalyzer};
use super::processing::database::{MpsDatabaseQuerier, MpsSQLiteExecutor};
#[cfg(feature = "mpd")]
use super::processing::database::{MpsMpdQuerier, MpsMpdExecutor};
use super::processing::general::{
    MpsFilesystemExecutor, MpsFilesystemQuerier, MpsOpStorage, MpsVariableStorer,
};
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug)]
pub struct MpsContext {
    pub database: Box<dyn MpsDatabaseQuerier>,
    pub variables: Box<dyn MpsVariableStorer>,
    pub filesystem: Box<dyn MpsFilesystemQuerier>,
    #[cfg(feature = "advanced")]
    pub analysis: Box<dyn MpsMusicAnalyzer>,
    #[cfg(feature = "mpd")]
    pub mpd_database: Box<dyn MpsMpdQuerier>,
}

impl Default for MpsContext {
    fn default() -> Self {
        Self {
            database: Box::new(MpsSQLiteExecutor::default()),
            variables: Box::new(MpsOpStorage::default()),
            filesystem: Box::new(MpsFilesystemExecutor::default()),
            #[cfg(feature = "advanced")]
            analysis: Box::new(MpsDefaultAnalyzer::default()),
            #[cfg(feature = "mpd")]
            mpd_database: Box::new(MpsMpdExecutor::default()),
        }
    }
}

impl Display for MpsContext {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "MpsContext{{...}}")?;
        Ok(())
    }
}
