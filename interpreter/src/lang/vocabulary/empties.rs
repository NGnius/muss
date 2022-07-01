use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::Token;
use crate::Context;

use crate::lang::{Lookup, LanguageDictionary, PseudoOp};
use crate::lang::{FunctionFactory, FunctionStatementFactory, IteratorItem, Op};
use crate::lang::{RuntimeError, RuntimeOp, SyntaxError};
use crate::processing::general::Type;
use crate::Item;

#[derive(Debug)]
pub struct EmptiesStatement {
    count: Lookup,
    context: Option<Context>,
    // state
    current_i: u64,
    is_errored: bool,
}

impl Display for EmptiesStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "empties({})", self.count)
    }
}

impl std::clone::Clone for EmptiesStatement {
    fn clone(&self) -> Self {
        Self {
            count: self.count.clone(),
            context: None,
            current_i: self.current_i,
            is_errored: self.is_errored,
        }
    }
}

impl Iterator for EmptiesStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.count.get(self.context.as_ref().unwrap());
        match val {
            Ok(val) => {
                if let Type::Primitive(val) = val {
                    if let Some(val) = val.clone().to_u64() {
                        if self.current_i < val {
                            self.current_i += 1;
                            Some(Ok(Item::new()))
                        } else {
                            None
                        }
                    } else {
                        self.is_errored = true;
                        Some(Err(RuntimeError {
                            line: 0,
                            op: PseudoOp::from_printable(self),
                            msg: format!(
                                "Cannot use primitive {} ({}) as count (should be UInt)",
                                self.count, val
                            ),
                        }))
                    }
                } else {
                    self.is_errored = true;
                    Some(Err(RuntimeError {
                        line: 0,
                        op: PseudoOp::from_printable(self),
                        msg: format!(
                            "Cannot use non-primitive {} ({}) as count (should be UInt)",
                            self.count, val
                        ),
                    }))
                }
            }
            Err(e) => {
                if self.is_errored {
                    None
                } else {
                    self.is_errored = true;
                    Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl Op for EmptiesStatement {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.current_i = 0;
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            count: self.count.clone(),
            context: None,
            current_i: 0,
            is_errored: false,
        })
    }
}

pub struct EmptiesFunctionFactory;

impl FunctionFactory<EmptiesStatement> for EmptiesFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "empties"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<EmptiesStatement, SyntaxError> {
        // empties(count)
        let count_lookup = Lookup::parse(tokens)?;
        Ok(EmptiesStatement {
            count: count_lookup,
            context: None,
            current_i: 0,
            is_errored: false,
        })
    }
}

pub type EmptiesStatementFactory =
    FunctionStatementFactory<EmptiesStatement, EmptiesFunctionFactory>;

#[inline(always)]
pub fn empties_function_factory() -> EmptiesStatementFactory {
    EmptiesStatementFactory::new(EmptiesFunctionFactory)
}
