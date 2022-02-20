use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token_raw, check_name, assert_name};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct EmptyItemOp;

impl Deref for EmptyItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for EmptyItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "empty()")
    }
}

impl MpsItemOp for EmptyItemOp {
    fn execute(&self, _context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        Ok(MpsType::empty())
    }
}

pub struct EmptyItemOpFactory;

impl MpsItemOpFactory<EmptyItemOp> for EmptyItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() == 3
        && check_name("empty", &tokens[0])
        && tokens[1].is_open_bracket()
        && tokens[2].is_close_bracket()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _factory: &MpsItemBlockFactory,
        _dict: &MpsLanguageDictionary,
    ) -> Result<EmptyItemOp, SyntaxError> {
        assert_name("empty", tokens)?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        assert_token_raw(MpsToken::CloseBracket, tokens)?;
        Ok(EmptyItemOp)
    }
}
