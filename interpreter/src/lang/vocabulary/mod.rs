#![allow(clippy::while_let_on_iterator)]
mod empties;
pub(crate) mod empty;
mod files;
mod intersection;
mod mpd_query;
mod playlist;
mod repeat;
mod reset;
mod sql_init;
mod sql_query;
mod sql_simple_query;
mod union;
mod variable_assign;
mod variable_iter;

pub use empties::{empties_function_factory, EmptiesStatementFactory};
pub use empty::{empty_function_factory, EmptyStatementFactory};
pub use files::{files_function_factory, FilesStatementFactory};
pub use intersection::{intersection_function_factory, IntersectionStatementFactory};
pub use mpd_query::{mpd_query_function_factory, MpdQueryStatementFactory};
pub use playlist::{playlist_function_factory, PlaylistStatementFactory};
pub use repeat::{repeat_function_factory, RepeatStatementFactory};
pub use reset::{reset_function_factory, ResetStatementFactory};
pub use sql_init::{sql_init_function_factory, SqlInitStatementFactory};
pub use sql_query::{sql_function_factory, SqlStatementFactory};
pub use sql_simple_query::{simple_sql_function_factory, SimpleSqlStatementFactory};
pub use union::{union_function_factory, UnionStatementFactory};
pub use variable_assign::{AssignStatement, AssignStatementFactory};
pub use variable_iter::{VariableRetrieveStatement, VariableRetrieveStatementFactory};

pub mod filters;
pub mod item_ops;
pub mod sorters;
