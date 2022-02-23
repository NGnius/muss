use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::vocabulary::filters::utility::{assert_comparison_operator, comparison_op};
use crate::lang::{MpsItemBlockFactory, MpsItemOp, MpsItemOpFactory};
use crate::lang::{MpsLanguageDictionary, MpsTypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct CompareItemOp {
    comparison: [i8; 2],
    lhs: Box<dyn MpsItemOp>,
    rhs: Box<dyn MpsItemOp>,
}

impl Deref for CompareItemOp {
    type Target = dyn MpsItemOp;
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

impl MpsItemOp for CompareItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let lhs_val = self.lhs.execute(context)?;
        let rhs_val = self.rhs.execute(context)?;
        if let MpsType::Primitive(lhs) = lhs_val {
            if let MpsType::Primitive(rhs) = rhs_val {
                let compare = lhs.compare(&rhs).map_err(|e| RuntimeMsg(e))?;
                let mut is_match = false;
                for comparator in self.comparison {
                    if comparator == compare {
                        is_match = true;
                        break;
                    }
                }
                Ok(MpsType::Primitive(MpsTypePrimitive::Bool(is_match)))
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

impl MpsItemOpFactory<CompareItemOp> for CompareItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        find_first_comparison(tokens).is_some()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
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

fn find_first_comparison(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut curly_depth = 0;
    let mut bracket_depth = 0;
    for i in 0..tokens.len() {
        match &tokens[i] {
            MpsToken::OpenCurly => curly_depth += 1,
            MpsToken::CloseCurly => {
                if curly_depth != 0 {
                    curly_depth -= 1;
                }
            }
            MpsToken::OpenBracket => bracket_depth += 1,
            MpsToken::CloseBracket => {
                if bracket_depth != 0 {
                    curly_depth -= 1;
                }
            }
            MpsToken::OpenAngleBracket | MpsToken::CloseAngleBracket => {
                if curly_depth == 0 && bracket_depth == 0 {
                    return Some(i);
                }
            }
            MpsToken::Equals | MpsToken::Exclamation => {
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
