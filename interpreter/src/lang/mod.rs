#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::needless_range_loop)]
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

pub use dictionary::LanguageDictionary;
pub(crate) use error::LanguageError;
pub use error::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
pub use filter::{FilterFactory, FilterPredicate, FilterStatement, FilterStatementFactory};
pub use filter_replace::FilterReplaceStatement;
pub use function::{FunctionFactory, FunctionStatementFactory};
pub use iter_block::{ItemBlockFactory, ItemOp, ItemOpFactory};
pub use lookup::Lookup;
pub use operation::{BoxedOpFactory, IteratorItem, Op, OpFactory, SimpleOpFactory};
pub use pseudo_op::PseudoOp;
pub use repeated_meme::{repeated_tokens, RepeatedTokens};
pub use single_op::SingleItem;
pub use sorter::{SortStatement, SortStatementFactory, Sorter, SorterFactory};
//pub(crate) use statement::Statement;
pub use type_primitives::TypePrimitive;

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
