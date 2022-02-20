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
pub struct AndItemOp {
    lhs: Box<dyn MpsItemOp>,
    rhs: Box<dyn MpsItemOp>,
}

impl Deref for AndItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for AndItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} && {}", self.lhs, self.rhs)
    }
}

impl MpsItemOp for AndItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let lhs = self.lhs.execute(context)?;
        if let MpsType::Primitive(MpsTypePrimitive::Bool(lhs)) = lhs {
            if !lhs {
                // short-circuit
                return Ok(MpsType::Primitive(MpsTypePrimitive::Bool(false)));
            }
            let rhs = self.rhs.execute(context)?;
            if let MpsType::Primitive(MpsTypePrimitive::Bool(rhs)) = rhs {
                Ok(MpsType::Primitive(MpsTypePrimitive::Bool(rhs)))
            } else {
                Err(RuntimeMsg(format!("Cannot apply logical AND to right-hand side of `{}` ({}): not Bool type", self.rhs, rhs)))
            }
        } else {
            Err(RuntimeMsg(format!("Cannot apply logical AND to left-hand side of `{}` ({}): not Bool type", self.lhs, lhs)))
        }
    }
}

pub struct AndItemOpFactory;

impl MpsItemOpFactory<AndItemOp> for AndItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        if let Some(and_location) = first_and(tokens) {
            and_location != 0
        } else {
            false
        }
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<AndItemOp, SyntaxError> {
        let and_location = first_and(tokens).unwrap();
        let mut end_tokens = tokens.split_off(and_location);
        let lhs_op = factory.try_build_item_statement(tokens, dict)?;
        assert_token_raw(MpsToken::Ampersand, &mut end_tokens)?;
        assert_token_raw(MpsToken::Ampersand, &mut end_tokens)?;
        let rhs_op = factory.try_build_item_statement(&mut end_tokens, dict)?;
        Ok(AndItemOp {
            lhs: lhs_op,
            rhs: rhs_op,
        })
    }
}

fn first_and(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len()-1 {
        let token = &tokens[i];
        if token.is_ampersand() && bracket_depth == 0 && tokens[i+1].is_ampersand() {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        }
    }
    None
}
