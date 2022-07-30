use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_name, assert_token, assert_token_raw, check_name};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct RemoveItemOp {
    variable_name: String,
    field_name: Option<String>,
}

impl Deref for RemoveItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for RemoveItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(field_name) = &self.field_name {
            write!(f, "remove {}.{}", self.variable_name, field_name)
        } else {
            write!(f, "remove {}", self.variable_name)
        }
    }
}

impl ItemOp for RemoveItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        if let Some(field_name) = &self.field_name {
            let var = context.variables.get_mut(&self.variable_name)?;
            if let Type::Item(item) = var {
                item.remove_field(field_name);
                Ok(Type::empty())
            } else {
                Err(RuntimeMsg(format!(
                    "Cannot access field `{}` on variable `{}` ({} is not Item)",
                    field_name, &self.variable_name, var
                )))
            }
        } else {
            context.variables.remove(&self.variable_name)?;
            Ok(Type::empty())
        }
    }
}

pub struct RemoveItemOpFactory;

impl ItemOpFactory<RemoveItemOp> for RemoveItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        (tokens.len() == 2 || tokens.len() == 4) && check_name("remove", &tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        _factory: &ItemBlockFactory,
        _dict: &LanguageDictionary,
    ) -> Result<RemoveItemOp, SyntaxError> {
        assert_name("remove", tokens)?;
        let name = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("variable_name".into()),
            tokens,
        )?;
        let field_opt = if tokens.is_empty() {
            None
        } else {
            assert_token_raw(Token::Dot, tokens)?;
            Some(assert_token(
                |t| match t {
                    Token::Name(s) => Some(s),
                    _ => None,
                },
                Token::Name("field_name".into()),
                tokens,
            )?)
        };
        Ok(RemoveItemOp {
            variable_name: name,
            field_name: field_opt,
        })
    }
}
