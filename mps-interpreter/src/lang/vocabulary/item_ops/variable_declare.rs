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
pub struct VariableDeclareItemOp {
    variable_name: String,
    inner: Option<Box<dyn MpsItemOp>>,
}

impl Deref for VariableDeclareItemOp {
    type Target = dyn MpsItemOp;
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

impl MpsItemOp for VariableDeclareItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        if let Some(inner) = &self.inner {
            let mps_type = inner.execute(context)?;
            if !context.variables.exists(&self.variable_name) {
                context.variables.declare(&self.variable_name, mps_type)?;
            }
        } else {
            if !context.variables.exists(&self.variable_name) {
                context
                    .variables
                    .declare(&self.variable_name, MpsType::empty())?;
            }
        }
        Ok(MpsType::empty())
    }
}

pub struct VariableDeclareItemOpFactory;

impl MpsItemOpFactory<VariableDeclareItemOp> for VariableDeclareItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() > 2 && tokens[0].is_let() && tokens[1].is_name()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<VariableDeclareItemOp, SyntaxError> {
        assert_token_raw(MpsToken::Let, tokens)?;
        //assert_name("let", tokens)?;
        let var_name = assert_token(
            |t| match t {
                MpsToken::Name(s) => Some(s),
                _ => None,
            },
            MpsToken::Name("variable_name".into()),
            tokens,
        )?;
        let inner_op: Option<Box<dyn MpsItemOp>>;
        if !tokens.is_empty() {
            assert_token_raw(MpsToken::Equals, tokens)?;
            inner_op = Some(factory.try_build_item_statement(tokens, dict)?);
        } else {
            inner_op = None;
        }
        Ok(VariableDeclareItemOp {
            variable_name: var_name,
            inner: inner_op,
        })
    }
}
