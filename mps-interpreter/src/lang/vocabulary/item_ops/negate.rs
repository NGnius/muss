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
pub struct NegateItemOp {
    rhs: Box<dyn MpsItemOp>,
}

impl Deref for NegateItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for NegateItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "- {}", self.rhs)
    }
}

impl MpsItemOp for NegateItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let rhs = self.rhs.execute(context)?;
        if let MpsType::Primitive(rhs) = &rhs {
            Ok(MpsType::Primitive(rhs.try_negate().map_err(|e| RuntimeMsg(e))?))
        } else {
            Err(RuntimeMsg(format!("Cannot negate `{}` ({}): not primitive type", self.rhs, rhs)))
        }
    }
}

pub struct NegateItemOpFactory;

impl MpsItemOpFactory<NegateItemOp> for NegateItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() >= 2 && tokens[0].is_minus()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<NegateItemOp, SyntaxError> {
        assert_token_raw(MpsToken::Minus, tokens)?;
        let rhs_op = factory.try_build_item_statement(tokens, dict)?;
        Ok(NegateItemOp {
            rhs: rhs_op,
        })
    }
}
