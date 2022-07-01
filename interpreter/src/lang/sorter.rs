use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;

use crate::lang::utility::{assert_name, assert_token_raw, check_name};
use crate::lang::LanguageDictionary;
use crate::lang::{BoxedOpFactory, IteratorItem, Op, PseudoOp};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
use crate::tokens::Token;
use crate::Context;

const SORTER_ITEM_CACHE_SIZE: usize = 8;

pub trait Sorter: Clone + Debug + Display {
    fn sort<'a>(
        &mut self,
        iterator: &mut dyn Op,
        item_buf: &mut VecDeque<IteratorItem>,
    ) -> Result<(), RuntimeMsg>;

    fn reset(&mut self) {}
}

pub trait SorterFactory<S: Sorter + 'static> {
    fn is_sorter(&self, tokens: &VecDeque<&Token>) -> bool;

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<S, SyntaxError>;
}

#[derive(Debug)]
pub struct SortStatement<S: Sorter + 'static> {
    orderer: S,
    iterable: PseudoOp,
    // state
    item_cache: VecDeque<IteratorItem>,
}

impl<S: Sorter + 'static> std::clone::Clone for SortStatement<S> {
    fn clone(&self) -> Self {
        Self {
            orderer: self.orderer.clone(),
            iterable: self.iterable.clone(),
            item_cache: VecDeque::new(),
        }
    }
}

impl<S: Sorter + 'static> Display for SortStatement<S> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}~({})", self.iterable, self.orderer)
    }
}

impl<S: Sorter + 'static> Op for SortStatement<S> {
    fn enter(&mut self, ctx: Context) {
        self.iterable.try_real().unwrap().enter(ctx)
    }

    fn escape(&mut self) -> Context {
        self.iterable.try_real().unwrap().escape()
    }

    fn is_resetable(&self) -> bool {
        if let Ok(iter) = self.iterable.try_real_ref() {
            iter.is_resetable()
        } else {
            false
        }
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.item_cache.clear();
        self.orderer.reset();
        self.iterable.try_real()?.reset()
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            orderer: self.orderer.clone(),
            iterable: PseudoOp::from(self.iterable.try_real_ref().unwrap().dup()),
            item_cache: VecDeque::new(),
        })
    }
}

impl<S: Sorter + 'static> Iterator for SortStatement<S> {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let real_op = match self.iterable.try_real() {
            Ok(op) => op,
            Err(e) => return Some(Err(e)),
        };
        match self.orderer.sort(real_op.as_mut(), &mut self.item_cache) {
            Ok(_) => {}
            Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
        }
        self.item_cache.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterable
            .try_real_ref()
            .map(|x| x.size_hint())
            .unwrap_or((0, None))
    }
}

pub struct SortStatementFactory<S: Sorter + 'static, F: SorterFactory<S> + 'static> {
    sort_factory: F,
    idc: PhantomData<S>,
}

impl<S: Sorter + 'static, F: SorterFactory<S> + 'static> SortStatementFactory<S, F> {
    pub fn new(factory: F) -> Self {
        Self {
            sort_factory: factory,
            idc: PhantomData::<S>,
        }
    }
}

impl<S: Sorter + 'static, F: SorterFactory<S> + 'static> BoxedOpFactory
    for SortStatementFactory<S, F>
{
    fn is_op_boxed(&self, tokens: &VecDeque<Token>) -> bool {
        let tokens_len = tokens.len();
        if let Some(tilde_location) = last_tilde(tokens, 0) {
            // iterable~(sorter)
            if tokens_len > tilde_location + 2 {
                let tokens2: VecDeque<&Token> =
                    VecDeque::from_iter(tokens.range(tilde_location + 2..tokens_len - 1));
                tokens[tokens_len - 1].is_close_bracket() && self.sort_factory.is_sorter(&tokens2)
            } else {
                false
            }
        } else if let Some(dot_location) = last_dot_sort(tokens, 1) {
            // iterable.sort(sorter)
            if tokens_len > dot_location + 3 {
                let tokens2: VecDeque<&Token> =
                    VecDeque::from_iter(tokens.range(dot_location + 3..tokens_len - 1));
                tokens[tokens_len - 1].is_close_bracket() && self.sort_factory.is_sorter(&tokens2)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        let inner_op;
        if let Some(tilde_location) = last_tilde(tokens, 0) {
            let end_tokens = tokens.split_off(tilde_location);
            inner_op = dict.try_build_statement(tokens)?;
            tokens.extend(end_tokens);
            assert_token_raw(Token::Tilde, tokens)?;
        } else if let Some(dot_location) = last_dot_sort(tokens, 1) {
            let end_tokens = tokens.split_off(dot_location);
            inner_op = dict.try_build_statement(tokens)?;
            tokens.extend(end_tokens);
            assert_token_raw(Token::Dot, tokens)?;
            assert_name("sort", tokens)?;
        } else {
            return Err(SyntaxError {
                line: 0,
                token: Token::Name(".|~".into()),
                got: tokens.pop_front(),
            });
        }
        assert_token_raw(Token::OpenBracket, tokens)?;
        let end_tokens = tokens.split_off(tokens.len() - 1);
        let sorter = self.sort_factory.build_sorter(tokens, dict)?;
        tokens.extend(end_tokens);
        assert_token_raw(Token::CloseBracket, tokens)?;
        Ok(Box::new(SortStatement {
            orderer: sorter,
            iterable: inner_op.into(),
            item_cache: VecDeque::with_capacity(SORTER_ITEM_CACHE_SIZE),
        }))
    }
}

fn last_tilde(tokens: &VecDeque<Token>, target_depth: usize) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in (0..tokens.len()).rev() {
        let current_token = &tokens[i];
        if current_token.is_close_bracket() {
            bracket_depth += 1;
        } else if current_token.is_open_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        } else if current_token.is_tilde() && bracket_depth == target_depth {
            return Some(i);
        }
    }
    None
}

fn last_dot_sort(tokens: &VecDeque<Token>, target_depth: usize) -> Option<usize> {
    let mut bracket_depth = 0;
    let mut sort_found = false;
    let mut bracket_found = false;
    for i in (0..tokens.len()).rev() {
        let current_token = &tokens[i];
        if sort_found {
            return {
                if current_token.is_dot() {
                    Some(i)
                } else {
                    None
                }
            };
        } else if bracket_found {
            if check_name("sort", current_token) {
                sort_found = true;
            } else {
                bracket_found = false;
            }
        }
        if current_token.is_close_bracket() {
            bracket_depth += 1;
        } else if current_token.is_open_bracket() {
            if target_depth == bracket_depth {
                bracket_found = true;
            }
            if bracket_depth != 0 {
                bracket_depth -= 1;
            }
        }
    }
    None
}
