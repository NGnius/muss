use std::collections::VecDeque;
use std::convert::AsRef;

use crate::lang::utility::{assert_token_raw, assert_token_raw_back};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

pub struct BracketsItemOpFactory;

impl ItemOpFactory<Box<dyn ItemOp>> for BracketsItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        tokens.len() >= 2
            && tokens[0].is_open_bracket()
            && tokens[tokens.len() - 1].is_close_bracket()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn ItemOp>, SyntaxError> {
        assert_token_raw(Token::OpenBracket, tokens)?;
        assert_token_raw_back(Token::CloseBracket, tokens)?;
        factory.try_build_item_statement(tokens, dict)
    }
}

impl ItemOp for Box<dyn ItemOp> {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        // while this sort of looks like it's (infinitely) recursive, it's actually not
        self.as_ref().execute(context)
    }
}
