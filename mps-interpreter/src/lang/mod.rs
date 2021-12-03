mod db_items;
mod dictionary;
mod error;
mod operation;
mod sql_query;
//mod statement;
pub(crate) mod utility;

pub use dictionary::MpsLanguageDictionary;
pub use error::{SyntaxError, RuntimeError, MpsLanguageError};
pub use operation::{MpsOp, MpsOpFactory, BoxedMpsOpFactory};
//pub(crate) use statement::MpsStatement;

pub mod vocabulary {
    pub use super::sql_query::{SqlStatement, SqlStatementFactory};
}

pub mod db {
    pub use super::db_items::{DEFAULT_SQLITE_FILEPATH, generate_default_db, DatabaseObj, DbMusicItem, DbAlbumItem, DbArtistItem, DbMetaItem, DbGenreItem};
}

#[cfg(test)]
mod tests {
    #[test]
    fn db_build_test() -> rusqlite::Result<()> {
        super::db::generate_default_db()?;
        Ok(())
    }
}
