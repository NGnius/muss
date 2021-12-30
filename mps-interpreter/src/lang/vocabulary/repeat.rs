use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::MpsContext;
use crate::MpsMusicItem;
use crate::tokens::MpsToken;

use crate::lang::{RuntimeError, SyntaxError};
use crate::lang::{MpsOp, PseudoOp, MpsFunctionFactory, MpsFunctionStatementFactory};
use crate::lang::MpsLanguageDictionary;
use crate::lang::utility::{assert_token_raw, assert_token};

#[derive(Debug)]
pub struct RepeatStatement {
    inner_statement: PseudoOp,
    inner_done: bool,
    context: Option<MpsContext>,
    cache: Vec<MpsMusicItem>,
    cache_position: usize,
    repetitions: usize,
    loop_forever: bool,
}

impl Display for RepeatStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "repeat({})", self.inner_statement)
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
        }
    }
}

impl Iterator for RepeatStatement {
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.inner_done {
            let real_op = match self.inner_statement.try_real() {
                Err(e) => return Some(Err(e)),
                Ok(real) => real
            };
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
                        },
                        Err(e) => Some(Err(e))
                    }
                },
                None => {
                    self.inner_done = true;
                    self.context = Some(real_op.escape());
                },
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
                        msg: "Cannot repeat nothing".into()
                    }))
                } else {
                    None
                }
            } else {
                let music_item = self.cache[self.cache_position].clone();
                self.cache_position += 1;
                if self.cache_position == self.cache.len() {
                    if self.repetitions != 0 { self.repetitions -= 1; }
                    self.cache_position = 0;
                }
                Some(Ok(music_item))
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
        if tokens.len() != 0 { // repititions specified
            assert_token_raw(MpsToken::Comma, tokens)?;
            count = Some(assert_token(|t| match t {
                MpsToken::Name(n) => n.parse::<usize>().map(|d| d - 1).ok(),
                _ => None
            }, MpsToken::Name("usize".into()), tokens)?);
        }
        Ok(RepeatStatement {
            inner_statement: inner_statement.into(),
            inner_done: false,
            context: None,
            cache: Vec::new(),
            cache_position: 0,
            repetitions: count.unwrap_or(0),
            loop_forever: count.is_none()
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

pub type RepeatStatementFactory = MpsFunctionStatementFactory<RepeatStatement, RepeatFunctionFactory>;

#[inline(always)]
pub fn repeat_function_factory() -> RepeatStatementFactory {
    RepeatStatementFactory::new(RepeatFunctionFactory)
}
