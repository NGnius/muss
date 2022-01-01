use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use std::iter::Iterator;

use super::MpsLanguageDictionary;
use super::{RuntimeError, SyntaxError};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsMusicItem;

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

pub trait MpsOp: Iterator<Item = Result<MpsMusicItem, RuntimeError>> + Debug + Display {
    fn enter(&mut self, ctx: MpsContext);

    fn escape(&mut self) -> MpsContext;
}
