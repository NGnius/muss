use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::{utility::assert_token_raw, Lookup};
use crate::lang::{MpsFilterFactory, MpsFilterPredicate, MpsFilterStatementFactory};
use crate::lang::{MpsLanguageDictionary, MpsTypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

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

impl MpsFilterPredicate for IndexFilter {
    fn matches(&mut self, _item: &MpsItem, ctx: &mut MpsContext) -> Result<bool, RuntimeMsg> {
        let index: u64 = match self.index.get(ctx)? {
            MpsType::Primitive(val) => match val {
                MpsTypePrimitive::Int(i) => *i as u64,
                MpsTypePrimitive::UInt(u) => *u,
                MpsTypePrimitive::Float(f) => *f as u64,
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

impl MpsFilterFactory<IndexFilter> for IndexFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        (tokens.len() == 1 && Lookup::check_is(tokens[0]))
            || (tokens.len() == 2 && tokens[0].is_exclamation() && Lookup::check_is(tokens[1]))
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<IndexFilter, SyntaxError> {
        let is_inverted = if tokens[0].is_exclamation() {
            assert_token_raw(MpsToken::Exclamation, tokens)?;
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

pub type IndexFilterStatementFactory = MpsFilterStatementFactory<IndexFilter, IndexFilterFactory>;

#[inline(always)]
pub fn index_filter() -> IndexFilterStatementFactory {
    IndexFilterStatementFactory::new(IndexFilterFactory)
}
