use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::{MpsSorter, MpsSorterFactory, MpsSortStatementFactory};
use crate::lang::{MpsLanguageDictionary, MpsIteratorItem, MpsOp};
use crate::lang::{RuntimeError, SyntaxError};
use crate::tokens::MpsToken;

#[derive(Debug, Clone)]
pub struct EmptySorter;

impl MpsSorter for EmptySorter {
    fn sort(&mut self, iterator: &mut dyn MpsOp, item_buf: &mut VecDeque<MpsIteratorItem>) -> Result<(), RuntimeError> {
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

impl MpsSorterFactory<EmptySorter> for EmptySorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        tokens.len() == 0
    }

    fn build_sorter(
        &self,
        _tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<EmptySorter, SyntaxError> {
        Ok(EmptySorter)
    }
}

pub type EmptySorterStatementFactory = MpsSortStatementFactory<EmptySorter, EmptySorterFactory>;

#[inline(always)]
pub fn empty_sort() -> EmptySorterStatementFactory {
    EmptySorterStatementFactory::new(EmptySorterFactory)
}