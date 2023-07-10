use std::collections::VecDeque;

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOpFactory};
use crate::lang::SyntaxError;
use crate::tokens::Token;

use super::VariableRetrieveItemOp;

pub struct FieldRetrieveItemOpFactory;

impl ItemOpFactory<VariableRetrieveItemOp> for FieldRetrieveItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        tokens.len() > 2
            && tokens[0].is_name()
            && tokens[1].is_dot()
            && tokens[2].is_name()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        _factory: &ItemBlockFactory,
        _dict: &LanguageDictionary,
    ) -> Result<VariableRetrieveItemOp, SyntaxError> {
        let var_name = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("variable_name".into()),
            tokens,
        )?;
        assert_token_raw(Token::Dot, tokens)?;
        let field = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("field_name".into()),
            tokens,
        )?;
        Ok(VariableRetrieveItemOp {
            variable_name: var_name,
            field_name: Some(field),
        })
    }
}
