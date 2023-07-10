use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::assert_token_raw;
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct SubtractItemOp {
    lhs: Box<dyn ItemOp>,
    rhs: Box<dyn ItemOp>,
}

impl Deref for SubtractItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for SubtractItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} - {}", self.lhs, self.rhs)
    }
}

impl ItemOp for SubtractItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let lhs = self.lhs.execute(context)?;
        if let Type::Primitive(lhs) = &lhs {
            let rhs = self.rhs.execute(context)?;
            if let Type::Primitive(rhs) = &rhs {
                Ok(Type::Primitive(lhs.try_subtract(rhs).map_err(RuntimeMsg)?))
            } else {
                Err(RuntimeMsg(format!(
                    "Cannot subtract right-hand side `{}` ({}): not primitive type",
                    self.rhs, rhs
                )))
            }
        } else {
            Err(RuntimeMsg(format!(
                "Cannot subtract left-hand side `{}` ({}): not primitive type",
                self.lhs, lhs
            )))
        }
    }
}

pub struct SubtractItemOpFactory;

impl ItemOpFactory<SubtractItemOp> for SubtractItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        if let Some(minus_location) = first_minus(tokens) {
            minus_location != 0
        } else {
            false
        }
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<SubtractItemOp, SyntaxError> {
        let minus_location = first_minus(tokens).unwrap();
        let end_tokens = tokens.split_off(minus_location);
        let lhs_op = factory.try_build_item_statement(tokens, dict)?;
        tokens.extend(end_tokens);
        assert_token_raw(Token::Minus, tokens)?;
        let rhs_op = factory.try_build_item_statement(tokens, dict)?;
        Ok(SubtractItemOp {
            lhs: lhs_op,
            rhs: rhs_op,
        })
    }
}

fn first_minus(tokens: &VecDeque<Token>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len() {
        let token = &tokens[i];
        if token.is_minus() && bracket_depth == 0 {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        } else if token.is_comma() && bracket_depth == 0 {
            return None;
        }
    }
    None
}
