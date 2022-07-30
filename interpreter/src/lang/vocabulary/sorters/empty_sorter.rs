use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::{IteratorItem, LanguageDictionary, Op};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{SortStatementFactory, Sorter, SorterFactory};
use crate::tokens::Token;

#[derive(Debug, Clone, Default)]
pub struct EmptySorter;

impl Sorter for EmptySorter {
    fn sort(
        &mut self,
        iterator: &mut dyn Op,
        item_buf: &mut VecDeque<IteratorItem>,
    ) -> Result<(), RuntimeMsg> {
        if let Some(item) = iterator.next() {
            item_buf.push_back(item)
        }
        Ok(())
    }
}

impl Display for EmptySorter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[empty]")
    }
}

pub struct EmptySorterFactory;

impl SorterFactory<EmptySorter> for EmptySorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&Token>) -> bool {
        tokens.is_empty()
    }

    fn build_sorter(
        &self,
        _tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<EmptySorter, SyntaxError> {
        Ok(EmptySorter)
    }
}

pub type EmptySorterStatementFactory = SortStatementFactory<EmptySorter, EmptySorterFactory>;

#[inline(always)]
pub fn empty_sort() -> EmptySorterStatementFactory {
    EmptySorterStatementFactory::new(EmptySorterFactory)
}
