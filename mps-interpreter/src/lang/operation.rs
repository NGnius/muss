use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use std::iter::Iterator;

use super::MpsLanguageDictionary;
use super::PseudoOp;
use super::{RuntimeError, SyntaxError};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

pub trait SimpleMpsOpFactory<T: MpsOp + 'static> {
    fn is_op_simple(&self, tokens: &VecDeque<MpsToken>) -> bool;

    fn build_op_simple(&self, tokens: &mut VecDeque<MpsToken>) -> Result<T, SyntaxError>;
}

impl<T: MpsOp + 'static, X: SimpleMpsOpFactory<T> + 'static> MpsOpFactory<T> for X {
    fn is_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        self.is_op_simple(tokens)
    }

    fn build_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<T, SyntaxError> {
        self.build_op_simple(tokens)
    }
}

pub trait MpsOpFactory<T: MpsOp + 'static> {
    fn is_op(&self, tokens: &VecDeque<MpsToken>) -> bool;

    fn build_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<T, SyntaxError>;

    #[inline]
    fn build_box(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsOp>, SyntaxError> {
        Ok(Box::new(self.build_op(tokens, dict)?))
    }
}

pub trait BoxedMpsOpFactory {
    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsOp>, SyntaxError>;

    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool;
}

pub type MpsIteratorItem = Result<MpsItem, RuntimeError>;

pub trait MpsOp: Iterator<Item = MpsIteratorItem> + Debug + Display {
    fn enter(&mut self, ctx: MpsContext);

    fn escape(&mut self) -> MpsContext;

    fn is_resetable(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        #[cfg(debug_assertions)]
        if self.is_resetable() {
            panic!(
                "MpsOp reported that it can be reset but did not implement reset (op: {})",
                self
            )
        }
        Err(RuntimeError {
            line: 0,
            op: PseudoOp::Fake(format!("{}", self)),
            msg: "Op does not support reset()".to_string(),
        })
    }
}
