use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::assert_token_raw;
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct NotItemOp {
    rhs: Box<dyn MpsItemOp>,
}

impl Deref for NotItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for NotItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "! {}", self.rhs)
    }
}

impl MpsItemOp for NotItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let rhs = self.rhs.execute(context)?;
        if let MpsType::Primitive(rhs) = &rhs {
            Ok(MpsType::Primitive(rhs.try_not().map_err(|e| RuntimeMsg(e))?))
        } else {
            Err(RuntimeMsg(format!("Cannot apply logical NOT to `{}` ({}): not primitive type", self.rhs, rhs)))
        }
    }
}

pub struct NotItemOpFactory;

impl MpsItemOpFactory<NotItemOp> for NotItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() >= 2 && tokens[0].is_exclamation()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<NotItemOp, SyntaxError> {
        assert_token_raw(MpsToken::Exclamation, tokens)?;
        let rhs_op = factory.try_build_item_statement(tokens, dict)?;
        Ok(NotItemOp {
            rhs: rhs_op,
        })
    }
}
