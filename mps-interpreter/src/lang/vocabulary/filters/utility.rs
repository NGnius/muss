use std::collections::VecDeque;

use crate::lang::utility::assert_token_raw;
use crate::lang::SyntaxError;
use crate::tokens::MpsToken;

pub fn assert_comparison_operator(tokens: &mut VecDeque<MpsToken>) -> Result<[i8; 2], SyntaxError> {
    let token1 = tokens.pop_front().unwrap();
    match token1 {
        MpsToken::Equals => {
            if tokens.len() != 0 && tokens[0].is_equals() {
                // tokens: ==
                assert_token_raw(MpsToken::Equals, tokens)?;
                Ok([0, 0])
            } else {
                Err(SyntaxError {
                    line: 0,
                    token: MpsToken::Equals,
                    got: if tokens.len() != 0 {
                        Some(tokens[0].clone())
                    } else {
                        None
                    },
                })
            }
        }
        MpsToken::OpenAngleBracket => {
            if tokens.len() != 0 && tokens[0].is_equals() {
                // tokens: <=
                assert_token_raw(MpsToken::Equals, tokens)?;
                Ok([0, -1])
            } else {
                // token: <
                Ok([-1, -1])
            }
        }
        MpsToken::CloseAngleBracket => {
            if tokens.len() != 0 && tokens[0].is_equals() {
                // tokens: >=
                assert_token_raw(MpsToken::Equals, tokens)?;
                Ok([0, 1])
            } else {
                // token: >
                Ok([1, 1])
            }
        }
        MpsToken::Exclamation => {
            assert_token_raw(MpsToken::Equals, tokens)?;
            Ok([-1, 1])
        }
        _ => Err(SyntaxError {
            line: 0,
            token: MpsToken::Equals, // TODO this can be < > or =
            got: Some(token1),
        }),
    }
}
