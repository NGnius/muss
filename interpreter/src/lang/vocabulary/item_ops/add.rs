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
pub struct AddItemOp {
    lhs: Box<dyn ItemOp>,
    rhs: Box<dyn ItemOp>,
}

impl Deref for AddItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for AddItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} + {}", self.lhs, self.rhs)
    }
}

impl ItemOp for AddItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let lhs = self.lhs.execute(context)?;
        if let Type::Primitive(lhs) = &lhs {
            let rhs = self.rhs.execute(context)?;
            if let Type::Primitive(rhs) = &rhs {
                Ok(Type::Primitive(lhs.try_add(rhs).map_err(RuntimeMsg)?))
            } else {
                Err(RuntimeMsg(format!(
                    "Cannot add right-hand side `{}` ({}): not primitive type",
                    self.rhs, rhs
                )))
            }
        } else {
            Err(RuntimeMsg(format!(
                "Cannot add left-hand side `{}` ({}): not primitive type",
                self.lhs, lhs
            )))
        }
    }
}

pub struct AddItemOpFactory;

impl ItemOpFactory<AddItemOp> for AddItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        if let Some(plus_location) = first_plus(tokens) {
            plus_location != 0
        } else {
            false
        }
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<AddItemOp, SyntaxError> {
        let plus_location = first_plus(tokens).unwrap();
        let mut end_tokens = tokens.split_off(plus_location);
        let lhs_op = factory.try_build_item_statement(tokens, dict)?;
        assert_token_raw(Token::Plus, &mut end_tokens)?;
        let rhs_op = factory.try_build_item_statement(&mut end_tokens, dict)?;
        Ok(AddItemOp {
            lhs: lhs_op,
            rhs: rhs_op,
        })
    }
}

fn first_plus(tokens: &VecDeque<Token>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len() {
        let token = &tokens[i];
        if token.is_plus() && bracket_depth == 0 {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        }
    }
    None
}
