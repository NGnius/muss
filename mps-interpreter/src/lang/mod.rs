mod comment;
mod db_items;
mod dictionary;
mod error;
mod operation;
mod sql_query;
mod sql_simple_query;
//mod statement;
pub(crate) mod utility;

pub use dictionary::MpsLanguageDictionary;
pub use error::{MpsLanguageError, RuntimeError, SyntaxError};
pub use operation::{BoxedMpsOpFactory, MpsOp, MpsOpFactory, SimpleMpsOpFactory};
//pub(crate) use statement::MpsStatement;

pub mod vocabulary {
    pub use super::sql_query::{SqlStatement, SqlStatementFactory};
    pub use super::sql_simple_query::{SimpleSqlStatement, SimpleSqlStatementFactory};
    pub use super::comment::{CommentStatement, CommentStatementFactory};
}

pub mod db {
    pub use super::db_items::{
        generate_default_db, DatabaseObj, DbAlbumItem, DbArtistItem, DbGenreItem, DbMetaItem,
        DbMusicItem, DEFAULT_SQLITE_FILEPATH,
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn db_build_test() -> rusqlite::Result<()> {
        super::db::generate_default_db()?;
        Ok(())
    }
}
