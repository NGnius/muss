use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct VariableAssignItemOp {
    variable_name: String,
    inner: Box<dyn ItemOp>,
}

impl Deref for VariableAssignItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for VariableAssignItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} = {}", &self.variable_name, &self.inner)
    }
}

impl ItemOp for VariableAssignItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let mps_type = self.inner.execute(context)?;
        context.variables.assign(&self.variable_name, mps_type)?;
        Ok(Type::empty())
    }
}

pub struct VariableAssignItemOpFactory;

impl ItemOpFactory<VariableAssignItemOp> for VariableAssignItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        tokens.len() > 2 && tokens[0].is_name() && tokens[1].is_equals()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<VariableAssignItemOp, SyntaxError> {
        let var_name = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("variable_name".into()),
            tokens,
        )?;
        assert_token_raw(Token::Equals, tokens)?;
        let inner_op = factory.try_build_item_statement(tokens, dict)?;
        Ok(VariableAssignItemOp {
            variable_name: var_name,
            inner: inner_op,
        })
    }
}
