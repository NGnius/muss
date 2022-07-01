use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_name, assert_token_raw, check_name};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct EmptyItemOp;

impl Deref for EmptyItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for EmptyItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "empty()")
    }
}

impl ItemOp for EmptyItemOp {
    fn execute(&self, _context: &mut Context) -> Result<Type, RuntimeMsg> {
        Ok(Type::empty())
    }
}

pub struct EmptyItemOpFactory;

impl ItemOpFactory<EmptyItemOp> for EmptyItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        tokens.len() == 3
            && check_name("empty", &tokens[0])
            && tokens[1].is_open_bracket()
            && tokens[2].is_close_bracket()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        _factory: &ItemBlockFactory,
        _dict: &LanguageDictionary,
    ) -> Result<EmptyItemOp, SyntaxError> {
        assert_name("empty", tokens)?;
        assert_token_raw(Token::OpenBracket, tokens)?;
        assert_token_raw(Token::CloseBracket, tokens)?;
        Ok(EmptyItemOp)
    }
}
