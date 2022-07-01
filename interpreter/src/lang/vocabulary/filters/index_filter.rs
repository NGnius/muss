use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::{utility::assert_token_raw, Lookup};
use crate::lang::{FilterFactory, FilterPredicate, FilterStatementFactory};
use crate::lang::{LanguageDictionary, TypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;
use crate::Item;

#[derive(Debug, Clone)]
pub struct IndexFilter {
    index: Lookup,
    // state
    current: u64,
    complete: bool,
    is_opposite: bool,
}

impl Display for IndexFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

impl FilterPredicate for IndexFilter {
    fn matches(&mut self, _item: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg> {
        let index: u64 = match self.index.get(ctx)? {
            Type::Primitive(val) => match val {
                TypePrimitive::Int(i) => *i as u64,
                TypePrimitive::UInt(u) => *u,
                TypePrimitive::Float(f) => *f as u64,
                val => return Err(RuntimeMsg(format!("Cannot use {} as index", val))),
            },
            val => return Err(RuntimeMsg(format!("Cannot use {} as index", val))),
        };
        if self.current == index && !self.is_opposite {
            self.current += 1;
            self.complete = true;
            Ok(true)
        } else if self.current != index && self.is_opposite {
            self.current += 1;
            Ok(true)
        } else {
            self.current += 1;
            Ok(false)
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        self.current = 0;
        self.complete = false;
        Ok(())
    }
}

pub struct IndexFilterFactory;

impl FilterFactory<IndexFilter> for IndexFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&Token>) -> bool {
        (tokens.len() == 1 && Lookup::check_is(tokens[0]))
            || (tokens.len() == 2 && tokens[0].is_exclamation() && Lookup::check_is(tokens[1]))
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<IndexFilter, SyntaxError> {
        let is_inverted = if tokens[0].is_exclamation() {
            assert_token_raw(Token::Exclamation, tokens)?;
            true
        } else {
            false
        };
        let lookup = Lookup::parse(tokens)?;
        Ok(IndexFilter {
            index: lookup,
            current: 0,
            complete: false,
            is_opposite: is_inverted,
        })
    }
}

pub type IndexFilterStatementFactory = FilterStatementFactory<IndexFilter, IndexFilterFactory>;

#[inline(always)]
pub fn index_filter() -> IndexFilterStatementFactory {
    IndexFilterStatementFactory::new(IndexFilterFactory)
}
