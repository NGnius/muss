mod filesystem;
#[cfg(feature = "advanced")]
mod music_analysis;
#[cfg(feature = "mpd")]
mod mpd;
mod sql;
mod variables;

//pub type OpGetter = dyn FnMut() -> crate::lang::PseudoOp;

pub mod database {
    pub use super::sql::{MpsDatabaseQuerier, MpsSQLiteExecutor, QueryResult};
    #[cfg(feature = "mpd")]
    pub use super::mpd::{MpsMpdQuerier, MpsMpdExecutor};
}

pub mod general {
    pub use super::filesystem::{FileIter, MpsFilesystemExecutor, MpsFilesystemQuerier};
    pub use super::variables::{MpsOpStorage, MpsType, MpsVariableStorer};
}

#[cfg(feature = "advanced")]
pub mod advanced {
    pub use super::music_analysis::{MpsDefaultAnalyzer, MpsMusicAnalyzer};
}
