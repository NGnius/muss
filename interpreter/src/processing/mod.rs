mod filesystem;
#[cfg(feature = "mpd")]
mod mpd;
#[cfg(feature = "advanced")]
mod music_analysis;
mod sql;
mod variables;

//pub type OpGetter = dyn FnMut() -> crate::lang::PseudoOp;

pub mod database {
    #[cfg(feature = "mpd")]
    pub use super::mpd::{MpdExecutor, MpdQuerier};
    pub use super::sql::{DatabaseQuerier, QueryResult};
    #[cfg(feature = "sql")]
    pub use super::sql::{SQLiteExecutor};
    #[cfg(feature = "fakesql")]
    pub use super::sql::{SQLiteTranspileExecutor};
    #[cfg(all(not(feature = "fakesql"), not(feature = "sql")))]
    pub use super::sql::{SQLErrExecutor};
}

pub mod general {
    pub use super::filesystem::{FileIter, FilesystemExecutor, FilesystemQuerier};
    pub use super::variables::{OpStorage, Type, VariableStorer};
}

#[cfg(feature = "advanced")]
pub mod advanced {
    pub use super::music_analysis::{DefaultAnalyzer, MusicAnalyzer, MusicAnalyzerDistance};
}
