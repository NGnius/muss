use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::assert_token;
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct VariableRetrieveItemOp {
    pub(super) variable_name: String,
    pub(super) field_name: Option<String>,
}

impl Deref for VariableRetrieveItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for VariableRetrieveItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(field) = &self.field_name {
            write!(f, "{}.{}", &self.variable_name, field)
        } else {
            write!(f, "{}", &self.variable_name)
        }
    }
}

impl ItemOp for VariableRetrieveItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let var = context.variables.get(&self.variable_name)?;
        if let Some(field_name) = &self.field_name {
            if let Type::Item(item) = var {
                match item.field(field_name) {
                    Some(val) => Ok(Type::Primitive(val.clone())),
                    None => Err(RuntimeMsg(format!(
                        "Cannot access field `{}` on variable `{}` (field does not exist)",
                        field_name, self.variable_name
                    ))),
                }
            } else {
                Err(RuntimeMsg(format!(
                    "Cannot access field `{}` on variable `{}` ({} is not Item)",
                    field_name, self.variable_name, var
                )))
            }
        } else {
            match var {
                Type::Op(op) => Ok(Type::Op(op.dup())),
                Type::Primitive(x) => Ok(Type::Primitive(x.clone())),
                Type::Item(item) => Ok(Type::Item(item.clone())),
            }
        }
    }
}

pub struct VariableRetrieveItemOpFactory;

impl ItemOpFactory<VariableRetrieveItemOp> for VariableRetrieveItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        !tokens.is_empty() && tokens[0].is_name()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        _factory: &ItemBlockFactory,
        _dict: &LanguageDictionary,
    ) -> Result<VariableRetrieveItemOp, SyntaxError> {
        let var_name = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("variable_name".into()),
            tokens,
        )?;
        Ok(VariableRetrieveItemOp {
            variable_name: var_name,
            field_name: None,
        })
    }
}
