use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_name, assert_token_raw, check_name};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{MpsItemBlockFactory, MpsItemOp, MpsItemOpFactory, MpsTypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct BranchItemOp {
    condition: Box<dyn MpsItemOp>,
    inner_ifs: Vec<Box<dyn MpsItemOp>>,
    inner_elses: Vec<Box<dyn MpsItemOp>>,
}

impl Deref for BranchItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for BranchItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "if {} {{", self.condition)?;
        if self.inner_ifs.len() > 1 {
            write!(f, "\n")?;
        }
        for i in 0..self.inner_ifs.len() {
            write!(f, "{}", self.inner_ifs[i])?;
            if i != self.inner_ifs.len() - 1 {
                write!(f, ",\n")?;
            }
        }
        if self.inner_ifs.len() > 1 {
            write!(f, "\n")?;
        }
        write!(f, "}} else {{")?;
        if self.inner_elses.len() > 1 {
            write!(f, "\n")?;
        }
        for i in 0..self.inner_elses.len() {
            write!(f, "{}", self.inner_elses[i])?;
            if i != self.inner_elses.len() - 1 {
                write!(f, ",\n")?;
            }
        }
        if self.inner_elses.len() > 1 {
            write!(f, "\n")?;
        }
        write!(f, "}}")
    }
}

impl MpsItemOp for BranchItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let condition_val = self.condition.execute(context)?;
        if let MpsType::Primitive(MpsTypePrimitive::Bool(condition)) = condition_val {
            let mut last_result = None;
            if condition {
                for op in self.inner_ifs.iter() {
                    last_result = Some(op.execute(context)?);
                }
            } else {
                for op in self.inner_elses.iter() {
                    last_result = Some(op.execute(context)?);
                }
            }
            if let Some(result) = last_result {
                Ok(result)
            } else {
                Ok(MpsType::empty())
            }
        } else {
            Err(RuntimeMsg(format!(
                "Cannot use {} ({}) as if branch condition (should be Bool)",
                self.condition, condition_val
            )))
        }
    }
}

pub struct BranchItemOpFactory;

impl MpsItemOpFactory<BranchItemOp> for BranchItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        !tokens.is_empty() && check_name("if", &tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<BranchItemOp, SyntaxError> {
        assert_name("if", tokens)?;
        // if condition
        let condition_op;
        if let Some(curly_pos) = next_curly_open_bracket(tokens) {
            let end_tokens = tokens.split_off(curly_pos);
            condition_op = factory.try_build_item_statement(tokens, dict)?;
            tokens.extend(end_tokens);
        } else {
            return Err(SyntaxError {
                line: 0,
                token: MpsToken::OpenCurly,
                got: tokens.pop_front(),
            });
        }
        // if block
        assert_token_raw(MpsToken::OpenCurly, tokens)?;
        let next_close_curly;
        if let Some(curly_pos) = next_curly_close_bracket(tokens) {
            next_close_curly = curly_pos;
        } else {
            return Err(SyntaxError {
                line: 0,
                token: MpsToken::CloseCurly,
                got: tokens.pop_back(),
            });
        }
        let end_tokens = tokens.split_off(next_close_curly);
        let mut inner_if_ops = Vec::new();
        while !tokens.is_empty() {
            if let Some(next_comma) = find_next_comma(tokens) {
                let end_tokens = tokens.split_off(next_comma);
                inner_if_ops.push(factory.try_build_item_statement(tokens, dict)?);
                tokens.extend(end_tokens);
                assert_token_raw(MpsToken::Comma, tokens)?;
            } else {
                inner_if_ops.push(factory.try_build_item_statement(tokens, dict)?);
            }
        }
        tokens.extend(end_tokens);
        assert_token_raw(MpsToken::CloseCurly, tokens)?;
        if tokens.is_empty() {
            // else block is omitted
            Ok(BranchItemOp {
                condition: condition_op,
                inner_ifs: inner_if_ops,
                inner_elses: Vec::with_capacity(0),
            })
        } else {
            // else block
            assert_name("else", tokens)?;
            assert_token_raw(MpsToken::OpenCurly, tokens)?;
            let next_close_curly;
            if let Some(curly_pos) = next_curly_close_bracket(tokens) {
                next_close_curly = curly_pos;
            } else {
                return Err(SyntaxError {
                    line: 0,
                    token: MpsToken::CloseCurly,
                    got: tokens.pop_back(),
                });
            }
            let end_tokens = tokens.split_off(next_close_curly);
            let mut inner_else_ops = Vec::new();
            while !tokens.is_empty() {
                if let Some(next_comma) = find_next_comma(tokens) {
                    let end_tokens = tokens.split_off(next_comma);
                    inner_else_ops.push(factory.try_build_item_statement(tokens, dict)?);
                    tokens.extend(end_tokens);
                    assert_token_raw(MpsToken::Comma, tokens)?;
                } else {
                    inner_else_ops.push(factory.try_build_item_statement(tokens, dict)?);
                }
            }
            tokens.extend(end_tokens);
            assert_token_raw(MpsToken::CloseCurly, tokens)?;
            Ok(BranchItemOp {
                condition: condition_op,
                inner_ifs: inner_if_ops,
                inner_elses: inner_else_ops,
            })
        }
    }
}

fn next_curly_open_bracket(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    for i in 0..tokens.len() {
        match &tokens[i] {
            MpsToken::OpenBracket => bracket_depth += 1,
            MpsToken::CloseBracket => {
                if bracket_depth != 0 {
                    bracket_depth -= 1;
                }
            }
            MpsToken::OpenCurly => {
                if bracket_depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn next_curly_close_bracket(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    let mut curly_depth = 0;
    for i in 0..tokens.len() {
        match &tokens[i] {
            MpsToken::OpenBracket => bracket_depth += 1,
            MpsToken::CloseBracket => {
                if bracket_depth != 0 {
                    bracket_depth -= 1;
                }
            }
            MpsToken::OpenCurly => curly_depth += 1,
            MpsToken::CloseCurly => {
                if bracket_depth == 0 && curly_depth == 0 {
                    return Some(i);
                } else if curly_depth != 0 {
                    curly_depth -= 1;
                }
            }
            _ => {}
        }
    }
    None
}

fn find_next_comma(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    let mut curly_depth = 0;
    for i in 0..tokens.len() {
        let token = &tokens[i];
        if token.is_comma() && bracket_depth == 0 && curly_depth == 0 {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        } else if token.is_open_curly() {
            curly_depth += 1;
        } else if token.is_close_curly() && curly_depth != 0 {
            curly_depth -= 1;
        }
    }
    None
}
