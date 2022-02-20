use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{check_name, assert_name};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory, MpsOp};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct IterItemOp {
    inner: Box<dyn MpsOp>,
}

impl Deref for IterItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for IterItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "iter {}", self.inner)
    }
}

impl MpsItemOp for IterItemOp {
    fn execute(&self, _context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        Ok(MpsType::Op(self.inner.dup().into()))
    }
}

pub struct IterItemOpFactory;

impl MpsItemOpFactory<IterItemOp> for IterItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        !tokens.is_empty()
        && check_name("iter", &tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<IterItemOp, SyntaxError> {
        assert_name("iter", tokens)?;
        let inner_op = dict.try_build_statement(tokens)?;
        Ok(IterItemOp {
            inner: inner_op,
        })
    }
}
