use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{MpsFunctionFactory, MpsFunctionStatementFactory, MpsOp, PseudoOp, MpsIteratorItem};
use crate::lang::{RuntimeError, SyntaxError};

#[derive(Debug)]
pub struct RepeatStatement {
    inner_statement: PseudoOp,
    inner_done: bool,
    context: Option<MpsContext>,
    cache: Vec<MpsItem>,
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
            write!(f, "repeat({}, {})", self.inner_statement, self.original_repetitions)
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
    type Item = MpsIteratorItem;

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
                while let Some(item) = real_op.next() {
                    return Some(item);
                }
                if !self.loop_forever {
                    if self.repetitions == 0 {
                        self.inner_done = true;
                        // take context from inner (should only occur when inner is no longer needed)
                        self.context = Some(real_op.escape());
                    } else {
                        self.repetitions -= 1;
                        match real_op.reset() {
                            Err(e) => return Some(Err(e)),
                            Ok(_) => {}
                        }
                    }
                } else {
                    // always reset in infinite loop mode
                    match real_op.reset() {
                        Err(e) => return Some(Err(e)),
                        Ok(_) => {}
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
            } else {
                if self.cache.len() == 0 {
                    if self.loop_forever {
                        Some(Err(RuntimeError {
                            line: 0,
                            op: (Box::new(self.clone()) as Box<dyn MpsOp>).into(),
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
    }
}

impl MpsOp for RepeatStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
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
        } else {
            if self.inner_done {
                self.repetitions = self.original_repetitions;
                self.cache_position = 0;
            } else {
                return Err(RuntimeError {
                    line: 0,
                    op: PseudoOp::from_printable(self),
                    msg:
                        "Cannot reset part way through repeat when inner statement is not resetable"
                            .to_string(),
                });
            }
        }
        Ok(())
    }
}

pub struct RepeatFunctionFactory;

impl MpsFunctionFactory<RepeatStatement> for RepeatFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "repeat"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<RepeatStatement, SyntaxError> {
        // repeat(query) or repeat(query, repetitions)
        let end_tokens = tokens.split_off(next_comma(tokens));
        let inner_statement = dict.try_build_statement(tokens)?;
        tokens.extend(end_tokens);
        let mut count: Option<usize> = None;
        let mut inner_done = false;
        if tokens.len() != 0 {
            // repititions specified
            assert_token_raw(MpsToken::Comma, tokens)?;
            count = Some(assert_token(
                |t| match t {
                    MpsToken::Name(n) => n
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
                MpsToken::Name("usize".into()),
                tokens,
            )?);
        }
        Ok(RepeatStatement {
            inner_statement: inner_statement.into(),
            inner_done: inner_done,
            context: None,
            cache: Vec::new(),
            cache_position: 0,
            repetitions: count.unwrap_or(0),
            loop_forever: count.is_none(),
            original_repetitions: count.and_then(|c| Some(c + 1)).unwrap_or(0),
        })
    }
}

fn next_comma(tokens: &VecDeque<MpsToken>) -> usize {
    for i in 0..tokens.len() {
        if tokens[i].is_comma() {
            return i;
        }
    }
    tokens.len()
}

pub type RepeatStatementFactory =
    MpsFunctionStatementFactory<RepeatStatement, RepeatFunctionFactory>;

#[inline(always)]
pub fn repeat_function_factory() -> RepeatStatementFactory {
    RepeatStatementFactory::new(RepeatFunctionFactory)
}
