use std::collections::VecDeque;
use std::path::PathBuf;

use super::MpsTypePrimitive;
use super::SyntaxError;
use crate::tokens::MpsToken;

pub fn assert_token<T, F: FnOnce(MpsToken) -> Option<T>>(
    caster: F,
    token: MpsToken,
    tokens: &mut VecDeque<MpsToken>,
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
            token: token,
            got: Some(result),
        })
    }
}

pub fn assert_token_raw(
    token: MpsToken,
    tokens: &mut VecDeque<MpsToken>,
) -> Result<MpsToken, SyntaxError> {
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
            token: token,
            got: Some(result),
        })
    }
}

pub fn assert_token_raw_back(
    token: MpsToken,
    tokens: &mut VecDeque<MpsToken>,
) -> Result<MpsToken, SyntaxError> {
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
            token: token,
            got: Some(result),
        })
    }
}

pub fn check_token_raw(token: MpsToken, token_target: &MpsToken) -> bool {
    std::mem::discriminant(&token) == std::mem::discriminant(token_target)
}

#[allow(dead_code)]
pub fn assert_name(name: &str, tokens: &mut VecDeque<MpsToken>) -> Result<String, SyntaxError> {
    let result = match tokens.pop_front() {
        Some(x) => Ok(x),
        None => Err(SyntaxError {
            line: 0,
            token: MpsToken::Name(name.to_owned()),
            got: None,
        }),
    }?;
    match result {
        MpsToken::Name(n) => {
            if n == name {
                Ok(n)
            } else {
                Err(SyntaxError {
                    line: 0,
                    token: MpsToken::Name(name.to_owned()),
                    got: Some(MpsToken::Name(n)),
                })
            }
        }
        token => Err(SyntaxError {
            line: 0,
            token: MpsToken::Name(name.to_owned()),
            got: Some(token),
        }),
    }
}

#[allow(dead_code)]
pub fn check_name(name: &str, token: &MpsToken) -> bool {
    match token {
        MpsToken::Name(n) => n == name,
        _ => false,
    }
}

pub fn check_is_type(token: &MpsToken) -> bool {
    match token {
        MpsToken::Literal(_) => true,
        MpsToken::Name(s) => {
            s.parse::<f64>().is_ok()
                || s.parse::<i64>().is_ok()
                || s.parse::<u64>().is_ok()
                || s == "false"
                || s == "true"
        }
        _ => false,
    }
}

pub fn assert_type(tokens: &mut VecDeque<MpsToken>) -> Result<MpsTypePrimitive, SyntaxError> {
    let token = match tokens.pop_front() {
        Some(x) => Ok(x),
        None => Err(SyntaxError {
            line: 0,
            token: MpsToken::Name("Float | UInt | Int | Bool".into()),
            got: None,
        }),
    }?;
    match token {
        MpsToken::Literal(s) => Ok(MpsTypePrimitive::String(s)),
        MpsToken::Name(s) => {
            if let Ok(f) = s.parse::<f64>() {
                Ok(MpsTypePrimitive::Float(f))
            } else if let Ok(i) = s.parse::<i64>() {
                Ok(MpsTypePrimitive::Int(i))
            } else if let Ok(u) = s.parse::<u64>() {
                Ok(MpsTypePrimitive::UInt(u))
            } else if s == "false" {
                Ok(MpsTypePrimitive::Bool(false))
            } else if s == "true" {
                Ok(MpsTypePrimitive::Bool(true))
            } else {
                Err(SyntaxError {
                    line: 0,
                    token: MpsToken::Name("Float | UInt | Int | Bool".into()),
                    got: Some(MpsToken::Name(s)),
                })
            }
        }
        token => Err(SyntaxError {
            line: 0,
            token: MpsToken::Name("Float | UInt | Int | Bool | \"String\"".into()),
            got: Some(token),
        }),
    }
}

pub fn music_folder() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("./"))
        .join("Music")
}
