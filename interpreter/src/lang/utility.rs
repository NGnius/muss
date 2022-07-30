use std::collections::VecDeque;
use std::path::PathBuf;

use super::SyntaxError;
use super::TypePrimitive;
use crate::tokens::Token;

pub fn assert_token<T, F: FnOnce(Token) -> Option<T>>(
    caster: F,
    token: Token,
    tokens: &mut VecDeque<Token>,
) -> Result<T, SyntaxError> {
    let result = match tokens.pop_front() {
        Some(x) => Ok(x),
        None => Err(SyntaxError {
            line: 0,
            token: token.clone(),
            got: None,
        }),
    }?;
    if let Some(out) = caster(result.clone()) {
        Ok(out)
    } else {
        Err(SyntaxError {
            line: 0,
            token,
            got: Some(result),
        })
    }
}

pub fn assert_token_raw(token: Token, tokens: &mut VecDeque<Token>) -> Result<Token, SyntaxError> {
    let result = match tokens.pop_front() {
        Some(x) => Ok(x),
        None => Err(SyntaxError {
            line: 0,
            token: token.clone(),
            got: None,
        }),
    }?;
    if std::mem::discriminant(&token) == std::mem::discriminant(&result) {
        Ok(result)
    } else {
        Err(SyntaxError {
            line: 0,
            token,
            got: Some(result),
        })
    }
}

pub fn assert_token_raw_back(
    token: Token,
    tokens: &mut VecDeque<Token>,
) -> Result<Token, SyntaxError> {
    let result = match tokens.pop_back() {
        Some(x) => Ok(x),
        None => Err(SyntaxError {
            line: 0,
            token: token.clone(),
            got: None,
        }),
    }?;
    if std::mem::discriminant(&token) == std::mem::discriminant(&result) {
        Ok(result)
    } else {
        Err(SyntaxError {
            line: 0,
            token,
            got: Some(result),
        })
    }
}

pub fn check_token_raw(token: Token, token_target: &Token) -> bool {
    std::mem::discriminant(&token) == std::mem::discriminant(token_target)
}

#[allow(dead_code)]
pub fn assert_name(name: &str, tokens: &mut VecDeque<Token>) -> Result<String, SyntaxError> {
    let result = match tokens.pop_front() {
        Some(x) => Ok(x),
        None => Err(SyntaxError {
            line: 0,
            token: Token::Name(name.to_owned()),
            got: None,
        }),
    }?;
    match result {
        Token::Name(n) => {
            if n == name {
                Ok(n)
            } else {
                Err(SyntaxError {
                    line: 0,
                    token: Token::Name(name.to_owned()),
                    got: Some(Token::Name(n)),
                })
            }
        }
        token => Err(SyntaxError {
            line: 0,
            token: Token::Name(name.to_owned()),
            got: Some(token),
        }),
    }
}

#[allow(dead_code)]
pub fn check_name(name: &str, token: &Token) -> bool {
    match token {
        Token::Name(n) => n == name,
        _ => false,
    }
}

pub fn check_is_type(token: &Token) -> bool {
    match token {
        Token::Literal(_) => true,
        Token::Name(s) => {
            s.parse::<i64>().is_ok()
                || s.parse::<u64>().is_ok()
                || s.parse::<f64>().is_ok()
                || s == "false"
                || s == "true"
        }
        _ => false,
    }
}

pub fn assert_type(tokens: &mut VecDeque<Token>) -> Result<TypePrimitive, SyntaxError> {
    let token = match tokens.pop_front() {
        Some(x) => Ok(x),
        None => Err(SyntaxError {
            line: 0,
            token: Token::Name("Float | UInt | Int | Bool".into()),
            got: None,
        }),
    }?;
    match token {
        Token::Literal(s) => Ok(TypePrimitive::String(s)),
        Token::Name(s) => {
            if let Ok(i) = s.parse::<i64>() {
                Ok(TypePrimitive::Int(i))
            } else if let Ok(u) = s.parse::<u64>() {
                Ok(TypePrimitive::UInt(u))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(TypePrimitive::Float(f))
            } else if s == "false" {
                Ok(TypePrimitive::Bool(false))
            } else if s == "true" {
                Ok(TypePrimitive::Bool(true))
            } else {
                Err(SyntaxError {
                    line: 0,
                    token: Token::Name("Float | UInt | Int | Bool".into()),
                    got: Some(Token::Name(s)),
                })
            }
        }
        token => Err(SyntaxError {
            line: 0,
            token: Token::Name("Float | UInt | Int | Bool | \"String\"".into()),
            got: Some(token),
        }),
    }
}

pub fn music_folder() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("./"))
        .join("Music")
}
