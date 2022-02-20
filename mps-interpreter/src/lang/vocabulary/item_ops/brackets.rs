use std::convert::AsRef;
use std::collections::VecDeque;

use crate::lang::utility::{assert_token_raw, assert_token_raw_back};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

pub struct BracketsItemOpFactory;

impl MpsItemOpFactory<Box<dyn MpsItemOp>> for BracketsItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() >= 2 && tokens[0].is_open_bracket() && tokens[tokens.len()-1].is_close_bracket()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsItemOp>, SyntaxError> {
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        assert_token_raw_back(MpsToken::CloseBracket, tokens)?;
        factory.try_build_item_statement(tokens, dict)
    }
}

impl MpsItemOp for Box<dyn MpsItemOp> {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        // while this sort of looks like it's (infinitely) recursive, it's actually not
        self.as_ref().execute(context)
    }
}
