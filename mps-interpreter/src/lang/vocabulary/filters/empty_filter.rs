use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::MpsContext;
use crate::MpsMusicItem;
use crate::tokens::MpsToken;
use crate::lang::{MpsFilterPredicate, MpsFilterFactory, MpsFilterStatementFactory};
use crate::lang::{SyntaxError, RuntimeError};
use crate::lang::MpsLanguageDictionary;
use crate::processing::OpGetter;

#[derive(Debug, Clone)]
pub struct EmptyFilter;

impl Display for EmptyFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[empty]")
    }
}

impl MpsFilterPredicate for EmptyFilter {
    fn matches(&mut self, _item: &MpsMusicItem, _ctx: &mut MpsContext, _op: &mut OpGetter) -> Result<bool, RuntimeError> {
        Ok(true)
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
