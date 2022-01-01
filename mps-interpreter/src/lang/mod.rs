mod db_items;
mod dictionary;
mod error;
mod filter;
mod function;
mod operation;
mod pseudo_op;
mod repeated_meme;
//mod statement;
mod type_primitives;
pub(crate) mod utility;

pub use dictionary::MpsLanguageDictionary;
pub use error::{MpsLanguageError, RuntimeError, SyntaxError};
pub use filter::{
    MpsFilterFactory, MpsFilterPredicate, MpsFilterStatement, MpsFilterStatementFactory,
};
pub use function::{MpsFunctionFactory, MpsFunctionStatementFactory};
pub use operation::{BoxedMpsOpFactory, MpsOp, MpsOpFactory, SimpleMpsOpFactory};
pub use pseudo_op::PseudoOp;
pub use repeated_meme::{repeated_tokens, RepeatedTokens};
//pub(crate) use statement::MpsStatement;
pub use type_primitives::MpsTypePrimitive;

pub mod vocabulary;

pub mod db {
    pub use super::db_items::{
        generate_db, generate_default_db, DatabaseObj, DbAlbumItem, DbArtistItem, DbGenreItem,
        DbMetaItem, DbMusicItem, DEFAULT_SQLITE_FILEPATH,
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
