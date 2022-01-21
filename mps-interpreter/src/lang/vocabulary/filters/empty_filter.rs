use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::MpsLanguageDictionary;
use crate::lang::{MpsFilterFactory, MpsFilterPredicate, MpsFilterStatementFactory};
use crate::lang::{RuntimeError, SyntaxError};
use crate::processing::OpGetter;
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

#[derive(Debug, Clone)]
pub struct EmptyFilter;

impl Display for EmptyFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[empty]")
    }
}

impl MpsFilterPredicate for EmptyFilter {
    fn matches(
        &mut self,
        _item: &MpsItem,
        _ctx: &mut MpsContext,
        _op: &mut OpGetter,
    ) -> Result<bool, RuntimeError> {
        Ok(true)
    }

    fn is_complete(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

pub struct EmptyFilterFactory;

impl MpsFilterFactory<EmptyFilter> for EmptyFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        tokens.len() == 0
    }

    fn build_filter(
        &self,
        _tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<EmptyFilter, SyntaxError> {
        Ok(EmptyFilter)
    }
}

pub type EmptyFilterStatementFactory = MpsFilterStatementFactory<EmptyFilter, EmptyFilterFactory>;

#[inline(always)]
pub fn empty_filter() -> EmptyFilterStatementFactory {
    EmptyFilterStatementFactory::new(EmptyFilterFactory)
}
