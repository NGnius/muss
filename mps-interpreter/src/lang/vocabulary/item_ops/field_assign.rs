use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{MpsItemBlockFactory, MpsItemOp, MpsItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct FieldAssignItemOp {
    variable_name: String,
    field_name: String,
    inner: Box<dyn MpsItemOp>,
}

impl Deref for FieldAssignItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for FieldAssignItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}.{} = {}",
            &self.variable_name, &self.field_name, &self.inner
        )
    }
}

impl MpsItemOp for FieldAssignItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let mps_type = self.inner.execute(context)?;
        let var = context.variables.get_mut(&self.variable_name)?;
        if let MpsType::Item(var) = var {
            if let MpsType::Primitive(val) = mps_type {
                var.set_field(&self.field_name, val);
                Ok(MpsType::empty())
            } else {
                Err(RuntimeMsg(format!(
                    "Cannot assign non-primitive {} to variable field `{}.{}`",
                    mps_type, &self.variable_name, &self.field_name
                )))
            }
        } else {
            Err(RuntimeMsg(format!(
                "Cannot access field `{}` on variable `{}` ({} is not Item)",
                &self.field_name, &self.variable_name, var
            )))
        }
    }
}

pub struct FieldAssignItemOpFactory;

impl MpsItemOpFactory<FieldAssignItemOp> for FieldAssignItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        (tokens.len() > 4
            && tokens[0].is_name()
            && tokens[1].is_dot()
            && tokens[2].is_name()
            && tokens[3].is_equals())
            || (tokens.len() > 3
                && tokens[0].is_dot()
                && tokens[1].is_name()
                && tokens[2].is_equals())
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<FieldAssignItemOp, SyntaxError> {
        let var_name;
        if tokens[0].is_dot() {
            var_name = "item".to_string();
        } else {
            var_name = assert_token(
                |t| match t {
                    MpsToken::Name(s) => Some(s),
                    _ => None,
                },
                MpsToken::Name("variable_name".into()),
                tokens,
            )?
        }
        assert_token_raw(MpsToken::Dot, tokens)?;
        let f_name = assert_token(
            |t| match t {
                MpsToken::Name(s) => Some(s),
                _ => None,
            },
            MpsToken::Name("field_name".into()),
            tokens,
        )?;
        assert_token_raw(MpsToken::Equals, tokens)?;
        let inner_op = factory.try_build_item_statement(tokens, dict)?;
        Ok(FieldAssignItemOp {
            variable_name: var_name,
            field_name: f_name,
            inner: inner_op,
        })
    }
}
