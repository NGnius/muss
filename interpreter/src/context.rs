#[cfg(feature = "advanced")]
use super::processing::advanced::{DefaultAnalyzer, MusicAnalyzer};
use super::processing::database::DatabaseQuerier;
#[cfg(feature = "fakesql")]
use super::processing::database::SQLiteTranspileExecutor;
#[cfg(feature = "mpd")]
use super::processing::database::{MpdExecutor, MpdQuerier};
use super::processing::general::{
    FilesystemExecutor, FilesystemQuerier, OpStorage, VariableStorer,
};
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug)]
pub struct Context {
    pub database: Box<dyn DatabaseQuerier>,
    pub variables: Box<dyn VariableStorer>,
    pub filesystem: Box<dyn FilesystemQuerier>,
    #[cfg(feature = "advanced")]
    pub analysis: Box<dyn MusicAnalyzer>,
    #[cfg(feature = "mpd")]
    pub mpd_database: Box<dyn MpdQuerier>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            #[cfg(feature = "fakesql")]
            database: Box::new(SQLiteTranspileExecutor::default()),
            #[cfg(all(feature = "sql", not(feature = "fakesql")))]
            database: Box::new(super::processing::database::SQLiteExecutor::default()),
            #[cfg(all(not(feature = "sql"), not(feature = "fakesql")))]
            database: Box::new(super::processing::database::SQLErrExecutor::default()),
            variables: Box::new(OpStorage::default()),
            filesystem: Box::new(FilesystemExecutor::default()),
            #[cfg(feature = "advanced")]
            analysis: Box::new(DefaultAnalyzer::default()),
            #[cfg(feature = "mpd")]
            mpd_database: Box::new(MpdExecutor::default()),
        }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Context{{...}}")?;
        Ok(())
    }
}
