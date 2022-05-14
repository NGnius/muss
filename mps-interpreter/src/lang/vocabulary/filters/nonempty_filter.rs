use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::MpsLanguageDictionary;
use crate::lang::{MpsFilterFactory, MpsFilterPredicate, MpsFilterStatementFactory};
use crate::lang::{RuntimeMsg, SyntaxError, utility::assert_token_raw};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

#[derive(Debug, Clone)]
pub struct NonEmptyFilter;

impl Display for NonEmptyFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[empty]")
    }
}

impl MpsFilterPredicate for NonEmptyFilter {
    fn matches(&mut self, item: &MpsItem, _ctx: &mut MpsContext) -> Result<bool, RuntimeMsg> {
        if item.len() != 0 {
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

impl MpsFilterFactory<NonEmptyFilter> for NonEmptyFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        tokens.len() >= 2 && tokens[0].is_interrogation() && tokens[1].is_interrogation()
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<NonEmptyFilter, SyntaxError> {
        assert_token_raw(MpsToken::Interrogation, tokens)?;
        assert_token_raw(MpsToken::Interrogation, tokens)?;
        Ok(NonEmptyFilter)
    }
}

pub type NonEmptyFilterStatementFactory = MpsFilterStatementFactory<NonEmptyFilter, NonEmptyFilterFactory>;

#[inline(always)]
pub fn nonempty_filter() -> NonEmptyFilterStatementFactory {
    NonEmptyFilterStatementFactory::new(NonEmptyFilterFactory)
}
