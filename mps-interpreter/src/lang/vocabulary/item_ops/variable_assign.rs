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
pub struct VariableAssignItemOp {
    variable_name: String,
    inner: Box<dyn MpsItemOp>,
}

impl Deref for VariableAssignItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for VariableAssignItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} = {}", &self.variable_name, &self.inner)
    }
}

impl MpsItemOp for VariableAssignItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let mps_type = self.inner.execute(context)?;
        context.variables.assign(&self.variable_name, mps_type)?;
        Ok(MpsType::empty())
    }
}

pub struct VariableAssignItemOpFactory;

impl MpsItemOpFactory<VariableAssignItemOp> for VariableAssignItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() > 2 && tokens[0].is_name() && tokens[1].is_equals()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<VariableAssignItemOp, SyntaxError> {
        let var_name = assert_token(|t| match t {
            MpsToken::Name(s) => Some(s),
            _ => None,
        }, MpsToken::Name("variable_name".into()), tokens)?;
        assert_token_raw(MpsToken::Equals, tokens)?;
        let inner_op = factory.try_build_item_statement(tokens, dict)?;
        Ok(VariableAssignItemOp {
            variable_name: var_name,
            inner: inner_op,
        })
    }
}
