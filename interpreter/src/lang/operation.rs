use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use std::iter::Iterator;

use super::LanguageDictionary;
use super::PseudoOp;
use super::{RuntimeError, SyntaxError};
use crate::tokens::Token;
use crate::Context;
use crate::Item;

// TODO change API to allow for accumulating modifiers
// e.g. build_op(&self, Option<Box<dyn Op>>, tokens) -> ...

pub trait SimpleOpFactory<T: Op + 'static> {
    fn is_op_simple(&self, tokens: &VecDeque<Token>) -> bool;

    fn build_op_simple(&self, tokens: &mut VecDeque<Token>) -> Result<T, SyntaxError>;
}

impl<T: Op + 'static, X: SimpleOpFactory<T> + 'static> OpFactory<T> for X {
    fn is_op(&self, tokens: &VecDeque<Token>) -> bool {
        self.is_op_simple(tokens)
    }

    fn build_op(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<T, SyntaxError> {
        self.build_op_simple(tokens)
    }
}

pub trait OpFactory<T: Op + 'static> {
    fn is_op(&self, tokens: &VecDeque<Token>) -> bool;

    fn build_op(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<T, SyntaxError>;

    #[inline]
    fn build_box(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        Ok(Box::new(self.build_op(tokens, dict)?))
    }
}

pub trait BoxedOpFactory: Send {
    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn Op>, SyntaxError>;

    fn is_op_boxed(&self, tokens: &VecDeque<Token>) -> bool;
}

pub type IteratorItem = Result<Item, RuntimeError>;

pub trait Op: Iterator<Item = IteratorItem> + Debug + Display + Send {
    fn enter(&mut self, ctx: Context);

    fn escape(&mut self) -> Context;

    fn is_resetable(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        #[cfg(debug_assertions)]
        if self.is_resetable() {
            panic!(
                "Op reported that it can be reset but did not implement reset (op: {})",
                self
            )
        }
        Err(RuntimeError {
            line: 0,
            op: PseudoOp::Fake(format!("{}", self)),
            msg: "Op does not support reset()".to_string(),
        })
    }

    // create an already-reset boxed clone of the op (without context)
    fn dup(&self) -> Box<dyn Op>;
}
