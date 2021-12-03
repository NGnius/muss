use std::iter::Iterator;
use std::collections::VecDeque;
use std::fmt::{Debug, Display};

use crate::MpsMusicItem;
use crate::MpsContext;
use crate::tokens::MpsToken;
use super::{SyntaxError, RuntimeError};

pub trait MpsOpFactory<T: MpsOp + 'static> {
    fn is_op(&self, tokens: &VecDeque<MpsToken>) -> bool;

    fn build_op(&self, tokens: &mut VecDeque<MpsToken>) -> Result<T, SyntaxError>;

    #[inline]
    fn build_box(&self, tokens: &mut VecDeque<MpsToken>) -> Result<Box<dyn MpsOp>, SyntaxError> {
        Ok(Box::new(self.build_op(tokens)?))
    }
}

pub trait BoxedMpsOpFactory {
    fn build_op_boxed(&self, tokens: &mut VecDeque<MpsToken>) -> Result<Box<dyn MpsOp>, SyntaxError>;

    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool;
}

pub trait MpsOp: Iterator<Item=Result<MpsMusicItem, RuntimeError>> + Debug + Display {
    fn enter(&mut self, ctx: MpsContext);

    fn escape(&mut self) -> MpsContext;
}
