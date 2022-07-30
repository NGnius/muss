use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::LanguageDictionary;
use crate::lang::{utility::assert_token_raw, RuntimeMsg, SyntaxError};
use crate::lang::{FilterFactory, FilterPredicate, FilterStatementFactory};
use crate::tokens::Token;
use crate::Context;
use crate::Item;

#[derive(Debug, Clone)]
pub struct NonEmptyFilter;

impl Display for NonEmptyFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[empty]")
    }
}

impl FilterPredicate for NonEmptyFilter {
    fn matches(&mut self, item: &Item, _ctx: &mut Context) -> Result<bool, RuntimeMsg> {
        if !item.is_empty() {
            if item.len() == 1 && item.field("filename").is_some() {
                Ok(false) // ignore filename field, since that almost always exists
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    fn is_complete(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        Ok(())
    }
}

pub struct NonEmptyFilterFactory;

impl FilterFactory<NonEmptyFilter> for NonEmptyFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&Token>) -> bool {
        tokens.len() >= 2 && tokens[0].is_interrogation() && tokens[1].is_interrogation()
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<NonEmptyFilter, SyntaxError> {
        assert_token_raw(Token::Interrogation, tokens)?;
        assert_token_raw(Token::Interrogation, tokens)?;
        Ok(NonEmptyFilter)
    }
}

pub type NonEmptyFilterStatementFactory =
    FilterStatementFactory<NonEmptyFilter, NonEmptyFilterFactory>;

#[inline(always)]
pub fn nonempty_filter() -> NonEmptyFilterStatementFactory {
    NonEmptyFilterStatementFactory::new(NonEmptyFilterFactory)
}
