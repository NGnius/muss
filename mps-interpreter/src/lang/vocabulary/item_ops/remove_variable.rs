use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token, check_name, assert_name, assert_token_raw};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct RemoveItemOp {
    variable_name: String,
    field_name: Option<String>
}

impl Deref for RemoveItemOp {
    type Target = dyn MpsItemOp;
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

impl MpsItemOp for RemoveItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        if let Some(field_name) = &self.field_name {
            let var = context.variables.get_mut(&self.variable_name)?;
            if let MpsType::Item(item) = var {
                item.remove_field(field_name);
                Ok(MpsType::empty())
            } else {
                Err(RuntimeMsg(format!("Cannot access field `{}` on variable `{}` ({} is not Item)", field_name, &self.variable_name, var)))
            }
        } else {
            context.variables.remove(&self.variable_name)?;
            Ok(MpsType::empty())
        }
    }
}

pub struct RemoveItemOpFactory;

impl MpsItemOpFactory<RemoveItemOp> for RemoveItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        (tokens.len() == 2 || tokens.len() == 4)&& check_name("remove", &tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _factory: &MpsItemBlockFactory,
        _dict: &MpsLanguageDictionary,
    ) -> Result<RemoveItemOp, SyntaxError> {
        assert_name("remove", tokens)?;
        let name = assert_token(|t| match t {
            MpsToken::Name(s) => Some(s),
            _ => None,
        }, MpsToken::Name("variable_name".into()), tokens)?;
        let field_opt;
        if tokens.is_empty() {
            field_opt = None;
        } else {
            assert_token_raw(MpsToken::Dot, tokens)?;
            field_opt = Some(assert_token(|t| match t {
                MpsToken::Name(s) => Some(s),
                _ => None,
            }, MpsToken::Name("field_name".into()), tokens)?);
        }
        Ok(RemoveItemOp {
            variable_name: name,
            field_name: field_opt,
        })
    }
}
