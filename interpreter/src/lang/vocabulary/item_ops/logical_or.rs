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
pub struct OrItemOp {
    lhs: Box<dyn ItemOp>,
    rhs: Box<dyn ItemOp>,
}

impl Deref for OrItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for OrItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} || {}", self.lhs, self.rhs)
    }
}

impl ItemOp for OrItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let lhs = self.lhs.execute(context)?;
        if let Type::Primitive(TypePrimitive::Bool(lhs)) = lhs {
            if lhs {
                // short-circuit
                return Ok(Type::Primitive(TypePrimitive::Bool(true)));
            }
            let rhs = self.rhs.execute(context)?;
            if let Type::Primitive(TypePrimitive::Bool(rhs)) = rhs {
                Ok(Type::Primitive(TypePrimitive::Bool(rhs)))
            } else {
                Err(RuntimeMsg(format!(
                    "Cannot apply logical OR to right-hand side of `{}` ({}): not Bool type",
                    self.rhs, rhs
                )))
            }
        } else {
            Err(RuntimeMsg(format!(
                "Cannot apply logical OR to left-hand side of `{}` ({}): not Bool type",
                self.lhs, lhs
            )))
        }
    }
}

pub struct OrItemOpFactory;

impl ItemOpFactory<OrItemOp> for OrItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        if let Some(or_location) = first_or(tokens) {
            or_location != 0
        } else {
            false
        }
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<OrItemOp, SyntaxError> {
        let or_location = first_or(tokens).unwrap();
        let mut end_tokens = tokens.split_off(or_location);
        let lhs_op = factory.try_build_item_statement(tokens, dict)?;
        assert_token_raw(Token::Pipe, &mut end_tokens)?;
        assert_token_raw(Token::Pipe, &mut end_tokens)?;
        let rhs_op = factory.try_build_item_statement(&mut end_tokens, dict)?;
        Ok(OrItemOp {
            lhs: lhs_op,
            rhs: rhs_op,
        })
    }
}

fn first_or(tokens: &VecDeque<Token>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len() - 1 {
        let token = &tokens[i];
        if token.is_pipe() && bracket_depth == 0 && tokens[i + 1].is_pipe() {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        }
    }
    None
}
