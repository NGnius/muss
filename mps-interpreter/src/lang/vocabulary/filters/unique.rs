use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Display, Error, Formatter};

use super::field_filter::FieldFilterErrorHandling;
use crate::lang::utility::{assert_name, assert_token, assert_token_raw, check_name};
use crate::lang::{MpsFilterFactory, MpsFilterPredicate, MpsFilterStatementFactory};
use crate::lang::{MpsLanguageDictionary, MpsTypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

#[derive(Debug, Clone)]
pub struct UniqueFieldFilter {
    field: String,
    field_errors: FieldFilterErrorHandling,
    // state
    seen: HashSet<MpsTypePrimitive>,
}

impl Display for UniqueFieldFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "unique {}", &self.field)
    }
}

impl MpsFilterPredicate for UniqueFieldFilter {
    fn matches(&mut self, item: &MpsItem, _ctx: &mut MpsContext) -> Result<bool, RuntimeMsg> {
        if let Some(field) = item.field(&self.field) {
            if self.seen.contains(field) {
                Ok(false)
            } else {
                self.seen.insert(field.to_owned());
                Ok(true)
            }
        } else {
            match self.field_errors {
                FieldFilterErrorHandling::Error => {
                    Err(RuntimeMsg(format!("Field {} does not exist", &self.field)))
                }
                FieldFilterErrorHandling::Ignore => Ok(false),
                FieldFilterErrorHandling::Include => Ok(true),
            }
        }
    }

    fn is_complete(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        self.seen.clear();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UniqueFilter {
    // state
    seen: HashSet<MpsItem>,
}

impl Display for UniqueFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "unique")
    }
}

impl MpsFilterPredicate for UniqueFilter {
    fn matches(&mut self, item: &MpsItem, _ctx: &mut MpsContext) -> Result<bool, RuntimeMsg> {
        if self.seen.contains(item) {
            Ok(false)
        } else {
            self.seen.insert(item.clone());
            Ok(true)
        }
    }

    fn is_complete(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        self.seen.clear();
        Ok(())
    }
}

pub struct UniqueFilterFactory;

impl MpsFilterFactory<UniqueFieldFilter> for UniqueFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        (tokens.len() == 2 || tokens.len() == 3)
            && check_name("unique", &tokens[0])
            && tokens[1].is_name()
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<UniqueFieldFilter, SyntaxError> {
        assert_name("unique", tokens)?;
        let field_name = assert_token(
            |t| match t {
                MpsToken::Name(s) => Some(s),
                _ => None,
            },
            MpsToken::Name("field_name".into()),
            tokens,
        )?;
        let error_handling = if !tokens.is_empty() {
            if tokens[0].is_exclamation() {
                assert_token_raw(MpsToken::Exclamation, tokens)?;
                FieldFilterErrorHandling::Ignore
            } else {
                assert_token_raw(MpsToken::Interrogation, tokens)?;
                FieldFilterErrorHandling::Include
            }
        } else {
            FieldFilterErrorHandling::Error
        };
        Ok(UniqueFieldFilter {
            field: field_name,
            field_errors: error_handling,
            seen: HashSet::new(),
        })
    }
}

impl MpsFilterFactory<UniqueFilter> for UniqueFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        tokens.len() == 1 && check_name("unique", &tokens[0])
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<UniqueFilter, SyntaxError> {
        assert_name("unique", tokens)?;
        Ok(UniqueFilter {
            seen: HashSet::new(),
        })
    }
}

pub type UniqueFieldFilterStatementFactory =
    MpsFilterStatementFactory<UniqueFieldFilter, UniqueFilterFactory>;

#[inline(always)]
pub fn unique_field_filter() -> UniqueFieldFilterStatementFactory {
    UniqueFieldFilterStatementFactory::new(UniqueFilterFactory)
}

pub type UniqueFilterStatementFactory =
    MpsFilterStatementFactory<UniqueFilter, UniqueFilterFactory>;

#[inline(always)]
pub fn unique_filter() -> UniqueFilterStatementFactory {
    UniqueFilterStatementFactory::new(UniqueFilterFactory)
}
