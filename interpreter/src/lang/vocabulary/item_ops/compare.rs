use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::vocabulary::filters::utility::{assert_comparison_operator, comparison_op};
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{LanguageDictionary, TypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct CompareItemOp {
    comparison: [i8; 2],
    lhs: Box<dyn ItemOp>,
    rhs: Box<dyn ItemOp>,
}

impl Deref for CompareItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for CompareItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{} {} {}",
            self.lhs,
            comparison_op(&self.comparison),
            self.rhs
        )
    }
}

impl ItemOp for CompareItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let lhs_val = self.lhs.execute(context)?;
        let rhs_val = self.rhs.execute(context)?;
        if let Type::Primitive(lhs) = lhs_val {
            if let Type::Primitive(rhs) = rhs_val {
                let compare = lhs.compare(&rhs).map_err(RuntimeMsg)?;
                let mut is_match = false;
                for comparator in self.comparison {
                    if comparator == compare {
                        is_match = true;
                        break;
                    }
                }
                Ok(Type::Primitive(TypePrimitive::Bool(is_match)))
            } else {
                Err(RuntimeMsg(format!(
                    "Cannot compare non-primitive right-hand side {} ({})",
                    self.rhs, rhs_val
                )))
            }
        } else {
            Err(RuntimeMsg(format!(
                "Cannot compare non-primitive left-hand side {} ({})",
                self.lhs, lhs_val
            )))
        }
    }
}

pub struct CompareItemOpFactory;

impl ItemOpFactory<CompareItemOp> for CompareItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        find_first_comparison(tokens).is_some()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<CompareItemOp, SyntaxError> {
        let comparison_loc = find_first_comparison(tokens).unwrap();
        let end_tokens = tokens.split_off(comparison_loc);
        let lhs_op = factory.try_build_item_statement(tokens, dict)?;
        tokens.extend(end_tokens);
        let comparison_arr = assert_comparison_operator(tokens)?;
        let rhs_op = factory.try_build_item_statement(tokens, dict)?;
        Ok(CompareItemOp {
            comparison: comparison_arr,
            lhs: lhs_op,
            rhs: rhs_op,
        })
    }
}

fn find_first_comparison(tokens: &VecDeque<Token>) -> Option<usize> {
    let mut curly_depth = 0;
    let mut bracket_depth = 0;
    for i in 0..tokens.len() {
        match &tokens[i] {
            Token::OpenCurly => curly_depth += 1,
            Token::CloseCurly => {
                if curly_depth != 0 {
                    curly_depth -= 1;
                }
            }
            Token::OpenBracket => bracket_depth += 1,
            Token::CloseBracket => {
                if bracket_depth != 0 {
                    curly_depth -= 1;
                }
            }
            Token::OpenAngleBracket | Token::CloseAngleBracket => {
                if curly_depth == 0 && bracket_depth == 0 {
                    return Some(i);
                }
            }
            Token::Equals | Token::Exclamation => {
                if curly_depth == 0
                    && bracket_depth == 0
                    && i + 1 != tokens.len()
                    && tokens[i + 1].is_equals()
                {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
