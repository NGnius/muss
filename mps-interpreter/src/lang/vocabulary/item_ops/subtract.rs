use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::assert_token_raw;
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct SubtractItemOp {
    lhs: Box<dyn MpsItemOp>,
    rhs: Box<dyn MpsItemOp>,
}

impl Deref for SubtractItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for SubtractItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} - {}", self.lhs, self.rhs)
    }
}

impl MpsItemOp for SubtractItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let lhs = self.lhs.execute(context)?;
        if let MpsType::Primitive(lhs) = &lhs {
            let rhs = self.rhs.execute(context)?;
            if let MpsType::Primitive(rhs) = &rhs {
                Ok(MpsType::Primitive(lhs.try_subtract(rhs).map_err(|e| RuntimeMsg(e))?))
            } else {
                Err(RuntimeMsg(format!("Cannot subtract right-hand side `{}` ({}): not primitive type", self.rhs, rhs)))
            }
        } else {
            Err(RuntimeMsg(format!("Cannot subtract left-hand side `{}` ({}): not primitive type", self.lhs, lhs)))
        }
    }
}

pub struct SubtractItemOpFactory;

impl MpsItemOpFactory<SubtractItemOp> for SubtractItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        if let Some(minus_location) = first_minus(tokens) {
            minus_location != 0
        } else {
            false
        }
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<SubtractItemOp, SyntaxError> {
        let minus_location = first_minus(tokens).unwrap();
        let mut end_tokens = tokens.split_off(minus_location);
        let lhs_op = factory.try_build_item_statement(tokens, dict)?;
        assert_token_raw(MpsToken::Minus, &mut end_tokens)?;
        let rhs_op = factory.try_build_item_statement(&mut end_tokens, dict)?;
        Ok(SubtractItemOp {
            lhs: lhs_op,
            rhs: rhs_op,
        })
    }
}

fn first_minus(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len() {
        let token = &tokens[i];
        if token.is_minus() && bracket_depth == 0 {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        }
    }
    None
}
