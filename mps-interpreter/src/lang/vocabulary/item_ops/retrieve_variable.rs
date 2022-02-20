use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token_raw, assert_token};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct VariableRetrieveItemOp {
    variable_name: String,
    field_name: Option<String>,
}

impl Deref for VariableRetrieveItemOp {
    type Target = dyn MpsItemOp;
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

impl MpsItemOp for VariableRetrieveItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let var = context.variables.get(&self.variable_name)?;
        if let Some(field_name) = &self.field_name {
            if let MpsType::Item(item) = var {
                Ok(match item.field(field_name) {
                    Some(val) => MpsType::Primitive(val.clone()),
                    None => MpsType::empty(),
                })
            } else {
                Err(RuntimeMsg(format!("Cannot access field `{}` on variable `{}` ({} is not Item)", field_name, self.variable_name, var)))
            }
        } else {
            match var {
                MpsType::Op(op) => Ok(MpsType::Op(op.dup())),
                MpsType::Primitive(x) => Ok(MpsType::Primitive(x.clone())),
                MpsType::Item(item) => Ok(MpsType::Item(item.clone()))
            }
        }
    }
}

pub struct VariableRetrieveItemOpFactory;

impl MpsItemOpFactory<VariableRetrieveItemOp> for VariableRetrieveItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        (tokens.len() == 1 && tokens[0].is_name())
        ||
        (tokens.len() == 3 && tokens[0].is_name() && tokens[1].is_dot() && tokens[2].is_name())
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _factory: &MpsItemBlockFactory,
        _dict: &MpsLanguageDictionary,
    ) -> Result<VariableRetrieveItemOp, SyntaxError> {
        let var_name = assert_token(|t| match t {
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
        Ok(VariableRetrieveItemOp {
            variable_name: var_name,
            field_name: field_opt,
        })
    }
}
