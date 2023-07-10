use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;

use crate::lang::utility::assert_token_raw;
use crate::lang::LanguageDictionary;
use crate::lang::{IteratorItem, Op, PseudoOp, BoxedTransformOpFactory};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
use crate::tokens::Token;
use crate::Context;

const SORTER_ITEM_CACHE_SIZE: usize = 8;

pub trait Sorter: Clone + Debug + Display + Send + Sync {
    fn sort(
        &mut self,
        iterator: &mut dyn Op,
        item_buf: &mut VecDeque<IteratorItem>,
    ) -> Result<(), RuntimeMsg>;

    fn reset(&mut self) {}
}

pub trait SorterFactory<S: Sorter + 'static>: Send + Sync {
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

impl<S: Sorter + 'static, F: SorterFactory<S> + 'static> BoxedTransformOpFactory
    for SortStatementFactory<S, F>
{
    fn build_transform_op(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
        op: Box<dyn Op>,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        assert_token_raw(Token::Tilde, tokens)?;
        assert_token_raw(Token::OpenBracket, tokens)?;
        let end_tokens = tokens.split_off(tokens.len() - 1);
        let sorter = self.sort_factory.build_sorter(tokens, dict)?;
        tokens.extend(end_tokens);
        assert_token_raw(Token::CloseBracket, tokens)?;
        Ok(Box::new(SortStatement {
            orderer: sorter,
            iterable: op.into(),
            item_cache: VecDeque::with_capacity(SORTER_ITEM_CACHE_SIZE),
        }))
    }

    fn is_transform_op(&self, tokens: &VecDeque<Token>) -> bool {
        if tokens.len() > 2 {
            let tokens2: VecDeque<&Token> =
                    VecDeque::from_iter(tokens.range(2..));
            tokens[0].is_tilde() && self.sort_factory.is_sorter(&tokens2)
        } else {
            false
        }

    }
}
