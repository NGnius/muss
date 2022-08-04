use std::collections::VecDeque;

use crate::lang::utility::assert_token_raw;
use crate::lang::SyntaxError;
use crate::tokens::Token;

#[inline]
pub fn sanitise_string(s: &str) -> String {
    #[cfg(feature = "unidecode")]
    let s = unidecode::unidecode(s);
    s.replace(|c: char| c.is_whitespace() || c == '_' || c == '-', "")
        .replace(|c: char| !(c.is_whitespace() || c.is_alphanumeric()), "")
        .to_lowercase()
}

pub fn assert_comparison_operator(tokens: &mut VecDeque<Token>) -> Result<[i8; 2], SyntaxError> {
    let token1 = tokens.pop_front().unwrap();
    match token1 {
        Token::Equals => {
            if !tokens.is_empty() && tokens[0].is_equals() {
                // tokens: ==
                assert_token_raw(Token::Equals, tokens)?;
                Ok([0, 0])
            } else {
                Err(SyntaxError {
                    line: 0,
                    token: Token::Equals,
                    got: if !tokens.is_empty() {
                        Some(tokens[0].clone())
                    } else {
                        None
                    },
                })
            }
        }
        Token::OpenAngleBracket => {
            if !tokens.is_empty() && tokens[0].is_equals() {
                // tokens: <=
                assert_token_raw(Token::Equals, tokens)?;
                Ok([0, -1])
            } else {
                // token: <
                Ok([-1, -1])
            }
        }
        Token::CloseAngleBracket => {
            if !tokens.is_empty() && tokens[0].is_equals() {
                // tokens: >=
                assert_token_raw(Token::Equals, tokens)?;
                Ok([0, 1])
            } else {
                // token: >
                Ok([1, 1])
            }
        }
        Token::Exclamation => {
            assert_token_raw(Token::Equals, tokens)?;
            Ok([-1, 1])
        }
        _ => Err(SyntaxError {
            line: 0,
            token: Token::Equals, // TODO this can be < > or =
            got: Some(token1),
        }),
    }
}

#[inline(always)]
pub fn comparison_op(c: &[i8; 2]) -> &str {
    match c {
        [-1, -1] => "<",
        [0, 0] => "==",
        [1, 1] => ">",
        [0, -1] => "<=",
        [0, 1] => ">=",
        [-1, 1] => "!=",
        _ => "??",
    }
}
