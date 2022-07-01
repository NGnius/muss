mod filesystem;
#[cfg(feature = "advanced")]
mod music_analysis;
#[cfg(feature = "mpd")]
mod mpd;
mod sql;
mod variables;

//pub type OpGetter = dyn FnMut() -> crate::lang::PseudoOp;

pub mod database {
    pub use super::sql::{DatabaseQuerier, SQLiteExecutor, QueryResult};
    #[cfg(feature = "mpd")]
    pub use super::mpd::{MpdQuerier, MpdExecutor};
}

pub mod general {
    pub use super::filesystem::{FileIter, FilesystemExecutor, FilesystemQuerier};
    pub use super::variables::{OpStorage, Type, VariableStorer};
}

#[cfg(feature = "advanced")]
pub mod advanced {
    pub use super::music_analysis::{DefaultAnalyzer, MusicAnalyzer};
}
