use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::Token;
use crate::Context;
use crate::Item;

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::LanguageDictionary;
use crate::lang::{
    FunctionFactory, FunctionStatementFactory, IteratorItem, Op, PseudoOp,
};
use crate::lang::{RuntimeError, SyntaxError};

#[derive(Debug)]
pub struct RepeatStatement {
    inner_statement: PseudoOp,
    inner_done: bool,
    context: Option<Context>,
    cache: Vec<Item>,
    cache_position: usize,
    repetitions: usize,
    loop_forever: bool,
    original_repetitions: usize,
}

impl Display for RepeatStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.loop_forever {
            write!(f, "repeat({})", self.inner_statement)
        } else {
            write!(
                f,
                "repeat({}, {})",
                self.inner_statement, self.original_repetitions
            )
        }
    }
}

impl std::clone::Clone for RepeatStatement {
    fn clone(&self) -> Self {
        Self {
            inner_statement: self.inner_statement.clone(),
            inner_done: self.inner_done,
            context: None,
            cache: self.cache.clone(),
            cache_position: self.cache_position,
            repetitions: self.repetitions,
            loop_forever: self.loop_forever,
            original_repetitions: self.original_repetitions,
        }
    }
}

impl Iterator for RepeatStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let real_op = match self.inner_statement.try_real() {
            Err(e) => return Some(Err(e)),
            Ok(real) => real,
        };
        // give context to inner (should only occur on first run)
        if self.context.is_some() {
            let ctx = self.context.take().unwrap();
            real_op.enter(ctx);
        }
        if real_op.is_resetable() {
            while self.loop_forever || !self.inner_done {
                if let Some(item) = real_op.next() {
                    return Some(item);
                }
                if !self.loop_forever {
                    if self.repetitions == 0 {
                        self.inner_done = true;
                        // take context from inner (should only occur when inner is no longer needed)
                        self.context = Some(real_op.escape());
                    } else {
                        self.repetitions -= 1;
                        if let Err(e) = real_op.reset() {
                            return Some(Err(e));
                        }
                    }
                } else {
                    // always reset in infinite loop mode
                    if let Err(e) = real_op.reset() {
                        return Some(Err(e));
                    }
                }
            }
            if self.context.is_none() {
                self.context = Some(real_op.escape());
            }
            None
        } else {
            // cache items in RepeatStatement since inner_statement cannot be reset
            if !self.inner_done {
                if self.context.is_some() {
                    let ctx = self.context.take().unwrap();
                    real_op.enter(ctx);
                }
                let inner_item = real_op.next();
                match inner_item {
                    Some(x) => {
                        return match x {
                            Ok(music) => {
                                self.cache.push(music.clone());
                                Some(Ok(music))
                            }
                            Err(e) => Some(Err(e)),
                        }
                    }
                    None => {
                        // inner has completed it's only run
                        self.inner_done = true;
                        self.context = Some(real_op.escape());
                    }
                }
            }
            // inner is done
            if self.repetitions == 0 && !self.loop_forever {
                None
            } else if self.cache.is_empty() {
                if self.loop_forever {
                    Some(Err(RuntimeError {
                        line: 0,
                        op: (Box::new(self.clone()) as Box<dyn Op>).into(),
                        msg: "Cannot repeat nothing".into(),
                    }))
                } else {
                    None
                }
            } else {
                let music_item = self.cache[self.cache_position].clone();
                self.cache_position += 1;
                if self.cache_position == self.cache.len() {
                    if self.repetitions != 0 {
                        self.repetitions -= 1;
                    }
                    self.cache_position = 0;
                }
                Some(Ok(music_item))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.inner_done {
            let len = (self.cache.len() * (self.repetitions + 1)) - self.cache_position;
            (len, Some(len))
        } else {
            (0, None)
        }
    }
}

impl Op for RepeatStatement {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        if self.context.is_some() {
            self.context.take().unwrap()
        } else {
            self.inner_statement.try_real().unwrap().escape()
        }
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        let real_op = self.inner_statement.try_real()?;
        if self.context.is_some() {
            let ctx = self.context.take().unwrap();
            real_op.enter(ctx);
        }
        if real_op.is_resetable() {
            real_op.reset()?;
            if self.original_repetitions == 0 {
                self.repetitions = 0;
                self.inner_done = true;
            } else {
                self.repetitions = self.original_repetitions - 1;
                self.inner_done = false;
            }
        } else if self.inner_done {
            self.repetitions = self.original_repetitions;
            self.cache_position = 0;
        } else {
            return Err(RuntimeError {
                line: 0,
                op: PseudoOp::from_printable(self),
                msg: "Cannot reset part way through repeat when inner statement is not resetable"
                    .to_string(),
            });
        }
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        let clone = Self {
            inner_statement: PseudoOp::from(self.inner_statement.try_real_ref().unwrap().dup()),
            inner_done: self.original_repetitions == 0,
            context: None,
            cache: Vec::new(),
            cache_position: 0,
            repetitions: if self.original_repetitions != 0 {
                self.original_repetitions - 1
            } else {
                0
            },
            loop_forever: self.loop_forever,
            original_repetitions: self.original_repetitions,
        };
        //clone.reset().unwrap();
        Box::new(clone)
    }
}

pub struct RepeatFunctionFactory;

impl FunctionFactory<RepeatStatement> for RepeatFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "repeat"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<RepeatStatement, SyntaxError> {
        // repeat(query) or repeat(query, repetitions)
        let end_tokens = tokens.split_off(next_comma(tokens));
        let inner_statement = dict.try_build_statement(tokens)?;
        tokens.extend(end_tokens);
        let mut count: Option<usize> = None;
        let mut inner_done = false;
        if !tokens.is_empty() {
            // repititions specified
            assert_token_raw(Token::Comma, tokens)?;
            count = Some(assert_token(
                |t| match t {
                    Token::Name(n) => n
                        .parse::<usize>()
                        .map(|d| {
                            if d == 0 {
                                inner_done = true;
                                0
                            } else {
                                d - 1
                            }
                        })
                        .ok(),
                    _ => None,
                },
                Token::Name("usize".into()),
                tokens,
            )?);
        }
        Ok(RepeatStatement {
            inner_statement: inner_statement.into(),
            inner_done,
            context: None,
            cache: Vec::new(),
            cache_position: 0,
            repetitions: count.unwrap_or(0),
            loop_forever: count.is_none(),
            original_repetitions: count.map(|c| c + 1).unwrap_or(0),
        })
    }
}

fn next_comma(tokens: &VecDeque<Token>) -> usize {
    for i in 0..tokens.len() {
        if tokens[i].is_comma() {
            return i;
        }
    }
    tokens.len()
}

pub type RepeatStatementFactory =
    FunctionStatementFactory<RepeatStatement, RepeatFunctionFactory>;

#[inline(always)]
pub fn repeat_function_factory() -> RepeatStatementFactory {
    RepeatStatementFactory::new(RepeatFunctionFactory)
}
