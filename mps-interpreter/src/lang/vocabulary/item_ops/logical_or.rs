use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::assert_token_raw;
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory, MpsTypePrimitive};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct OrItemOp {
    lhs: Box<dyn MpsItemOp>,
    rhs: Box<dyn MpsItemOp>,
}

impl Deref for OrItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for OrItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} || {}", self.lhs, self.rhs)
    }
}

impl MpsItemOp for OrItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let lhs = self.lhs.execute(context)?;
        if let MpsType::Primitive(MpsTypePrimitive::Bool(lhs)) = lhs {
            if lhs {
                // short-circuit
                return Ok(MpsType::Primitive(MpsTypePrimitive::Bool(true)));
            }
            let rhs = self.rhs.execute(context)?;
            if let MpsType::Primitive(MpsTypePrimitive::Bool(rhs)) = rhs {
                Ok(MpsType::Primitive(MpsTypePrimitive::Bool(rhs)))
            } else {
                Err(RuntimeMsg(format!("Cannot apply logical OR to right-hand side of `{}` ({}): not Bool type", self.rhs, rhs)))
            }
        } else {
            Err(RuntimeMsg(format!("Cannot apply logical OR to left-hand side of `{}` ({}): not Bool type", self.lhs, lhs)))
        }
    }
}

pub struct OrItemOpFactory;

impl MpsItemOpFactory<OrItemOp> for OrItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        if let Some(or_location) = first_or(tokens) {
            or_location != 0
        } else {
            false
        }
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<OrItemOp, SyntaxError> {
        let or_location = first_or(tokens).unwrap();
        let mut end_tokens = tokens.split_off(or_location);
        let lhs_op = factory.try_build_item_statement(tokens, dict)?;
        assert_token_raw(MpsToken::Pipe, &mut end_tokens)?;
        assert_token_raw(MpsToken::Pipe, &mut end_tokens)?;
        let rhs_op = factory.try_build_item_statement(&mut end_tokens, dict)?;
        Ok(OrItemOp {
            lhs: lhs_op,
            rhs: rhs_op,
        })
    }
}

fn first_or(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len()-1 {
        let token = &tokens[i];
        if token.is_pipe() && bracket_depth == 0 && tokens[i+1].is_pipe() {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        }
    }
    None
}
