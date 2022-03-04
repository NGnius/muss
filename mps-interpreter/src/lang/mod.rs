mod db_items;
mod dictionary;
mod error;
mod filter;
mod filter_replace;
mod function;
mod iter_block;
mod lookup;
mod operation;
mod pseudo_op;
mod repeated_meme;
mod single_op;
mod sorter;
//mod statement;
mod type_primitives;
pub(crate) mod utility;

pub use dictionary::MpsLanguageDictionary;
pub use error::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
pub(crate) use error::MpsLanguageError;
pub use filter::{
    MpsFilterFactory, MpsFilterPredicate, MpsFilterStatement, MpsFilterStatementFactory,
};
pub use filter_replace::MpsFilterReplaceStatement;
pub use function::{MpsFunctionFactory, MpsFunctionStatementFactory};
pub use iter_block::{MpsItemBlockFactory, MpsItemOp, MpsItemOpFactory};
pub use lookup::Lookup;
pub use operation::{BoxedMpsOpFactory, MpsIteratorItem, MpsOp, MpsOpFactory, SimpleMpsOpFactory};
pub use pseudo_op::PseudoOp;
pub use repeated_meme::{repeated_tokens, RepeatedTokens};
pub use single_op::SingleItem;
pub use sorter::{MpsSortStatement, MpsSortStatementFactory, MpsSorter, MpsSorterFactory};
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
