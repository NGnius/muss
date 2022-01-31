mod filesystem;
mod sql;
mod variables;

//pub type OpGetter = dyn FnMut() -> crate::lang::PseudoOp;

pub mod database {
    pub use super::sql::{MpsDatabaseQuerier, MpsSQLiteExecutor, QueryResult};
}

pub mod general {
    pub use super::filesystem::{FileIter, MpsFilesystemExecutor, MpsFilesystemQuerier};
    pub use super::variables::{MpsOpStorage, MpsType, MpsVariableStorer};
}
