use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_name, check_name};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory, Op};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct IterItemOp {
    inner: Box<dyn Op>,
}

impl Deref for IterItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for IterItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "iter {}", self.inner)
    }
}

impl ItemOp for IterItemOp {
    fn execute(&self, _context: &mut Context) -> Result<Type, RuntimeMsg> {
        Ok(Type::Op(self.inner.dup()))
    }
}

pub struct IterItemOpFactory;

impl ItemOpFactory<IterItemOp> for IterItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        !tokens.is_empty() && check_name("iter", &tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        _factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<IterItemOp, SyntaxError> {
        assert_name("iter", tokens)?;
        let inner_op = dict.try_build_statement(tokens)?;
        Ok(IterItemOp { inner: inner_op })
    }
}
