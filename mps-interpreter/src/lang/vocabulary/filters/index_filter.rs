use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::{MpsLanguageDictionary, MpsTypePrimitive};
use crate::lang::{MpsFilterFactory, MpsFilterPredicate, MpsFilterStatementFactory};
use crate::lang::{RuntimeError, SyntaxError};
use crate::lang::Lookup;
use crate::processing::{OpGetter, general::MpsType};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsMusicItem;

#[derive(Debug, Clone)]
pub struct IndexFilter {
    index: Lookup,
    // state
    current: u64,
    complete: bool,
}

impl Display for IndexFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

impl MpsFilterPredicate for IndexFilter {
    fn matches(
        &mut self,
        _item: &MpsMusicItem,
        ctx: &mut MpsContext,
        op: &mut OpGetter,
    ) -> Result<bool, RuntimeError> {
        let index: u64 = match self.index.get(ctx, op)? {
            MpsType::Primitive(val) => match val {
                MpsTypePrimitive::Int(i) => *i as u64,
                MpsTypePrimitive::UInt(u) => *u,
                MpsTypePrimitive::Float(f) => *f as u64,
                val => return Err(RuntimeError {
                    line: 0,
                    op: op(),
                    msg: format!("Cannot use {} as index", val),
                })
            },
            val => return Err(RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Cannot use {} as index", val),
            })
        };
        if self.current == index {
            self.current += 1;
            self.complete = true;
            Ok(true)
        } else {
            self.current += 1;
            Ok(false)
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.current = 0;
        self.complete = false;
        Ok(())
    }
}

pub struct IndexFilterFactory;

impl MpsFilterFactory<IndexFilter> for IndexFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        tokens.len() == 1
        && Lookup::check_is(&tokens[0])
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<IndexFilter, SyntaxError> {
        let lookup = Lookup::parse(tokens)?;
        Ok(IndexFilter {
            index: lookup,
            current: 0,
            complete: false,
        })
    }
}

pub type IndexFilterStatementFactory = MpsFilterStatementFactory<IndexFilter, IndexFilterFactory>;

#[inline(always)]
pub fn index_filter() -> IndexFilterStatementFactory {
    IndexFilterStatementFactory::new(IndexFilterFactory)
}
