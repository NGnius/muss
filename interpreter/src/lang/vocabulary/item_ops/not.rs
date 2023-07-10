use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::assert_token_raw;
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct NotItemOp {
    rhs: Box<dyn ItemOp>,
}

impl Deref for NotItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for NotItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "! {}", self.rhs)
    }
}

impl ItemOp for NotItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let rhs = self.rhs.execute(context)?;
        if let Type::Primitive(rhs) = &rhs {
            Ok(Type::Primitive(rhs.try_not().map_err(RuntimeMsg)?))
        } else {
            Err(RuntimeMsg(format!(
                "Cannot apply logical NOT to `{}` ({}): not primitive type",
                self.rhs, rhs
            )))
        }
    }
}

pub struct NotItemOpFactory;

impl ItemOpFactory<NotItemOp> for NotItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        !tokens.is_empty() && tokens[0].is_exclamation()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<NotItemOp, SyntaxError> {
        assert_token_raw(Token::Exclamation, tokens)?;
        let rhs_op = factory.try_build_item_statement(tokens, dict)?;
        Ok(NotItemOp { rhs: rhs_op })
    }
}
