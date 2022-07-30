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
pub struct VariableDeclareItemOp {
    variable_name: String,
    inner: Option<Box<dyn ItemOp>>,
}

impl Deref for VariableDeclareItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for VariableDeclareItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(inner) = &self.inner {
            write!(f, "let {} = {}", &self.variable_name, inner)
        } else {
            write!(f, "let {}", &self.variable_name)
        }
    }
}

impl ItemOp for VariableDeclareItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        if let Some(inner) = &self.inner {
            let mps_type = inner.execute(context)?;
            if !context.variables.exists(&self.variable_name) {
                context.variables.declare(&self.variable_name, mps_type)?;
            }
        } else if !context.variables.exists(&self.variable_name) {
            context
                .variables
                .declare(&self.variable_name, Type::empty())?;
        }
        Ok(Type::empty())
    }
}

pub struct VariableDeclareItemOpFactory;

impl ItemOpFactory<VariableDeclareItemOp> for VariableDeclareItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        tokens.len() >= 2 && tokens[0].is_let() && tokens[1].is_name()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<VariableDeclareItemOp, SyntaxError> {
        assert_token_raw(Token::Let, tokens)?;
        //assert_name("let", tokens)?;
        let var_name = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("variable_name".into()),
            tokens,
        )?;
        let inner_op: Option<Box<dyn ItemOp>> = if !tokens.is_empty() {
            assert_token_raw(Token::Equals, tokens)?;
            Some(factory.try_build_item_statement(tokens, dict)?)
        } else {
            None
        };
        Ok(VariableDeclareItemOp {
            variable_name: var_name,
            inner: inner_op,
        })
    }
}
