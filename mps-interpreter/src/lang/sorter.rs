use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;

use crate::lang::utility::{assert_name, assert_token_raw, check_name};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{BoxedMpsOpFactory, MpsIteratorItem, MpsOp, PseudoOp};
use crate::lang::{RuntimeError, SyntaxError};
use crate::processing::OpGetter;
use crate::tokens::MpsToken;
use crate::MpsContext;

const SORTER_ITEM_CACHE_SIZE: usize = 8;

pub trait MpsSorter: Clone + Debug + Display {
    fn sort<'a>(
        &mut self,
        iterator: &mut dyn MpsOp,
        item_buf: &mut VecDeque<MpsIteratorItem>,
        op: &'a mut OpGetter,
    ) -> Result<(), RuntimeError>;

    fn reset(&mut self) {}
}

pub trait MpsSorterFactory<S: MpsSorter + 'static> {
    fn is_sorter(&self, tokens: &VecDeque<&MpsToken>) -> bool;

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<S, SyntaxError>;
}

#[derive(Debug)]
pub struct MpsSortStatement<S: MpsSorter + 'static> {
    orderer: S,
    iterable: PseudoOp,
    // state
    item_cache: VecDeque<MpsIteratorItem>,
}

impl<S: MpsSorter + 'static> std::clone::Clone for MpsSortStatement<S> {
    fn clone(&self) -> Self {
        Self {
            orderer: self.orderer.clone(),
            iterable: self.iterable.clone(),
            item_cache: VecDeque::new(),
        }
    }
}

impl<S: MpsSorter + 'static> Display for MpsSortStatement<S> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}~({})", self.iterable, self.orderer)
    }
}

impl<S: MpsSorter + 'static> MpsOp for MpsSortStatement<S> {
    fn enter(&mut self, ctx: MpsContext) {
        self.iterable.try_real().unwrap().enter(ctx)
    }

    fn escape(&mut self) -> MpsContext {
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
}

impl<S: MpsSorter + 'static> Iterator for MpsSortStatement<S> {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let pseudo_self = PseudoOp::from_printable(self);
        let real_op = match self.iterable.try_real() {
            Ok(op) => op,
            Err(e) => return Some(Err(e)),
        };
        match self
            .orderer
            .sort(real_op.as_mut(), &mut self.item_cache, &mut move || {
                pseudo_self.clone()
            }) {
            Ok(_) => {}
            Err(e) => return Some(Err(e)),
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

pub struct MpsSortStatementFactory<S: MpsSorter + 'static, F: MpsSorterFactory<S> + 'static> {
    sort_factory: F,
    idc: PhantomData<S>,
}

impl<S: MpsSorter + 'static, F: MpsSorterFactory<S> + 'static> MpsSortStatementFactory<S, F> {
    pub fn new(factory: F) -> Self {
        Self {
            sort_factory: factory,
            idc: PhantomData::<S>,
        }
    }
}

impl<S: MpsSorter + 'static, F: MpsSorterFactory<S> + 'static> BoxedMpsOpFactory
    for MpsSortStatementFactory<S, F>
{
    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool {
        let tokens_len = tokens.len();
        if let Some(tilde_location) = last_tilde(tokens, 0) {
            // iterable~(sorter)
            if tokens_len > tilde_location + 2 {
                let tokens2: VecDeque<&MpsToken> =
                    VecDeque::from_iter(tokens.range(tilde_location + 2..tokens_len - 1));
                tokens[tokens_len - 1].is_close_bracket() && self.sort_factory.is_sorter(&tokens2)
            } else {
                false
            }
        } else if let Some(dot_location) = last_dot_sort(tokens, 1) {
            // iterable.sort(sorter)
            if tokens_len > dot_location + 3 {
                let tokens2: VecDeque<&MpsToken> =
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
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsOp>, SyntaxError> {
        let inner_op;
        if let Some(tilde_location) = last_tilde(tokens, 0) {
            let end_tokens = tokens.split_off(tilde_location);
            inner_op = dict.try_build_statement(tokens)?;
            tokens.extend(end_tokens);
            assert_token_raw(MpsToken::Tilde, tokens)?;
        } else if let Some(dot_location) = last_dot_sort(tokens, 1) {
            let end_tokens = tokens.split_off(dot_location);
            inner_op = dict.try_build_statement(tokens)?;
            tokens.extend(end_tokens);
            assert_token_raw(MpsToken::Dot, tokens)?;
            assert_name("sort", tokens)?;
        } else {
            return Err(SyntaxError {
                line: 0,
                token: MpsToken::Name(".|~".into()),
                got: tokens.pop_front(),
            });
        }
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        let end_tokens = tokens.split_off(tokens.len() - 1);
        let sorter = self.sort_factory.build_sorter(tokens, dict)?;
        tokens.extend(end_tokens);
        assert_token_raw(MpsToken::CloseBracket, tokens)?;
        Ok(Box::new(MpsSortStatement {
            orderer: sorter,
            iterable: inner_op.into(),
            item_cache: VecDeque::with_capacity(SORTER_ITEM_CACHE_SIZE),
        }))
    }
}

fn last_tilde(tokens: &VecDeque<MpsToken>, target_depth: usize) -> Option<usize> {
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

fn last_dot_sort(tokens: &VecDeque<MpsToken>, target_depth: usize) -> Option<usize> {
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
