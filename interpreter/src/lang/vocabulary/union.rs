use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::Token;
use crate::Context;

use crate::lang::repeated_tokens;
use crate::lang::{FunctionFactory, FunctionStatementFactory, IteratorItem, Op};
use crate::lang::{LanguageDictionary, PseudoOp};
use crate::lang::{RuntimeError, SyntaxError};

#[derive(Debug, Copy, Clone)]
enum UnionStrategy {
    Sequential,
    Interleave,
}

#[derive(Debug)]
pub struct UnionStatement {
    context: Option<Context>,
    ops: Vec<PseudoOp>,
    strategy: UnionStrategy,
    index: usize,
}

impl Display for UnionStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut ops_str = "".to_owned();
        for i in 0..self.ops.len() {
            ops_str += &self.ops[i].to_string();
            if i != self.ops.len() - 1 {
                ops_str += ", ";
            }
        }
        write!(f, "union({})", ops_str)
    }
}

impl std::clone::Clone for UnionStatement {
    fn clone(&self) -> Self {
        Self {
            context: None,
            ops: self.ops.clone(),
            strategy: self.strategy,
            index: self.index,
        }
    }
}

impl Iterator for UnionStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.ops.len() {
            return None;
        }
        match self.strategy {
            UnionStrategy::Sequential => loop {
                if self.index == self.ops.len() {
                    return None;
                }
                let real_op = match self.ops[self.index].try_real() {
                    Ok(x) => x,
                    Err(e) => return Some(Err(e)),
                };
                real_op.enter(self.context.take().unwrap());
                if let Some(item) = real_op.next() {
                    self.context = Some(real_op.escape());
                    return Some(item);
                }
                self.context = Some(real_op.escape());
                self.index += 1;
            },
            UnionStrategy::Interleave => {
                let mut none_count = 0;
                let ops_len = self.ops.len();
                loop {
                    if none_count == ops_len {
                        self.index = ops_len;
                        return None;
                    }
                    let real_op = match self.ops[self.index].try_real() {
                        Ok(x) => x,
                        Err(e) => return Some(Err(e)),
                    };
                    self.index += 1;
                    // loop back to beginning when at end
                    if self.index == ops_len {
                        self.index = 0;
                    }
                    real_op.enter(self.context.take().unwrap());
                    if let Some(item) = real_op.next() {
                        self.context = Some(real_op.escape());
                        return Some(item);
                    }
                    self.context = Some(real_op.escape());
                    none_count += 1;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl Op for UnionStatement {
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
        for op in &mut self.ops {
            let real_op = op.try_real()?;
            real_op.enter(self.context.take().unwrap());
            if real_op.is_resetable() {
                let result = real_op.reset();
                self.context = Some(real_op.escape());
                result?;
            } else {
                self.context = Some(real_op.escape());
            }
        }
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        let mut ops_clone = Vec::with_capacity(self.ops.len());
        for op in self.ops.iter() {
            ops_clone.push(PseudoOp::from(op.try_real_ref().unwrap().dup()))
        }
        Box::new(Self {
            context: None,
            ops: ops_clone,
            strategy: self.strategy,
            index: 0,
        })
    }
}

pub struct UnionFunctionFactory;

impl FunctionFactory<UnionStatement> for UnionFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "union" || name == "u" || name == "interleave" || name == "interlace"
    }

    fn build_function_params(
        &self,
        name: String,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<UnionStatement, SyntaxError> {
        // union(op1, op2, ...)
        let operations = repeated_tokens(
            |tokens| if tokens[0].is_close_bracket() {
                Ok(None)
            } else {
                Ok(Some(PseudoOp::from(dict.try_build_statement(tokens)?)))
            },
            Token::Comma,
        )
        .ingest_all(tokens)?;
        let combine_strategy = if name == "u" || name == "union" {
            UnionStrategy::Sequential
        } else {
            UnionStrategy::Interleave
        };
        Ok(UnionStatement {
            context: None,
            ops: operations,
            strategy: combine_strategy,
            index: 0,
        })
    }
}

pub type UnionStatementFactory = FunctionStatementFactory<UnionStatement, UnionFunctionFactory>;

#[inline(always)]
pub fn union_function_factory() -> UnionStatementFactory {
    UnionStatementFactory::new(UnionFunctionFactory)
}

pub(super) fn next_comma(tokens: &VecDeque<Token>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len() {
        let token = &tokens[i];
        if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() {
            bracket_depth -= 1;
        } else if token.is_comma() && bracket_depth == 0 {
            return Some(i);
        }
    }
    None
}
