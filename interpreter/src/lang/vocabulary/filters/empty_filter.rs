use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::LanguageDictionary;
use crate::lang::{FilterFactory, FilterPredicate, FilterStatementFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::tokens::Token;
use crate::Context;
use crate::Item;

#[derive(Debug, Clone)]
pub struct EmptyFilter;

impl Display for EmptyFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[empty]")
    }
}

impl FilterPredicate for EmptyFilter {
    fn matches(&mut self, _item: &Item, _ctx: &mut Context) -> Result<bool, RuntimeMsg> {
        Ok(true)
    }

    fn is_complete(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        Ok(())
    }
}

pub struct EmptyFilterFactory;

impl FilterFactory<EmptyFilter> for EmptyFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&Token>) -> bool {
        !tokens.is_empty() && tokens[0].is_close_bracket()
    }

    fn build_filter(
        &self,
        _tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<EmptyFilter, SyntaxError> {
        Ok(EmptyFilter)
    }
}

pub type EmptyFilterStatementFactory = FilterStatementFactory<EmptyFilter, EmptyFilterFactory>;

#[inline(always)]
pub fn empty_filter() -> EmptyFilterStatementFactory {
    EmptyFilterStatementFactory::new(EmptyFilterFactory)
}
