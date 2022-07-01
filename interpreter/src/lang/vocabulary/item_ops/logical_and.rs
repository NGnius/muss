use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::assert_token_raw;
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory, TypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct AndItemOp {
    lhs: Box<dyn ItemOp>,
    rhs: Box<dyn ItemOp>,
}

impl Deref for AndItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for AndItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} && {}", self.lhs, self.rhs)
    }
}

impl ItemOp for AndItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let lhs = self.lhs.execute(context)?;
        if let Type::Primitive(TypePrimitive::Bool(lhs)) = lhs {
            if !lhs {
                // short-circuit
                return Ok(Type::Primitive(TypePrimitive::Bool(false)));
            }
            let rhs = self.rhs.execute(context)?;
            if let Type::Primitive(TypePrimitive::Bool(rhs)) = rhs {
                Ok(Type::Primitive(TypePrimitive::Bool(rhs)))
            } else {
                Err(RuntimeMsg(format!(
                    "Cannot apply logical AND to right-hand side of `{}` ({}): not Bool type",
                    self.rhs, rhs
                )))
            }
        } else {
            Err(RuntimeMsg(format!(
                "Cannot apply logical AND to left-hand side of `{}` ({}): not Bool type",
                self.lhs, lhs
            )))
        }
    }
}

pub struct AndItemOpFactory;

impl ItemOpFactory<AndItemOp> for AndItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        if let Some(and_location) = first_and(tokens) {
            and_location != 0
        } else {
            false
        }
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<AndItemOp, SyntaxError> {
        let and_location = first_and(tokens).unwrap();
        let mut end_tokens = tokens.split_off(and_location);
        let lhs_op = factory.try_build_item_statement(tokens, dict)?;
        assert_token_raw(Token::Ampersand, &mut end_tokens)?;
        assert_token_raw(Token::Ampersand, &mut end_tokens)?;
        let rhs_op = factory.try_build_item_statement(&mut end_tokens, dict)?;
        Ok(AndItemOp {
            lhs: lhs_op,
            rhs: rhs_op,
        })
    }
}

fn first_and(tokens: &VecDeque<Token>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len() - 1 {
        let token = &tokens[i];
        if token.is_ampersand() && bracket_depth == 0 && tokens[i + 1].is_ampersand() {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        }
    }
    None
}
