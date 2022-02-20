use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{check_is_type, assert_type};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory, MpsTypePrimitive};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct ConstantItemOp {
    value: MpsTypePrimitive,
}

impl Deref for ConstantItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for ConstantItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.value)
    }
}

impl MpsItemOp for ConstantItemOp {
    fn execute(&self, _context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        Ok(MpsType::Primitive(self.value.clone()))
    }
}

pub struct ConstantItemOpFactory;

impl MpsItemOpFactory<ConstantItemOp> for ConstantItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() == 1
        && check_is_type(&tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _factory: &MpsItemBlockFactory,
        _dict: &MpsLanguageDictionary,
    ) -> Result<ConstantItemOp, SyntaxError> {
        let const_value = assert_type(tokens)?;
        Ok(ConstantItemOp {
            value: const_value,
        })
    }
}
