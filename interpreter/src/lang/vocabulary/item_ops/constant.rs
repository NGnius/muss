use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_type, check_is_type};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory, TypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct ConstantItemOp {
    value: TypePrimitive,
}

impl Deref for ConstantItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for ConstantItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.value)
    }
}

impl ItemOp for ConstantItemOp {
    fn execute(&self, _context: &mut Context) -> Result<Type, RuntimeMsg> {
        Ok(Type::Primitive(self.value.clone()))
    }
}

pub struct ConstantItemOpFactory;

impl ItemOpFactory<ConstantItemOp> for ConstantItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        tokens.len() == 1 && check_is_type(&tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        _factory: &ItemBlockFactory,
        _dict: &LanguageDictionary,
    ) -> Result<ConstantItemOp, SyntaxError> {
        let const_value = assert_type(tokens)?;
        Ok(ConstantItemOp { value: const_value })
    }
}
