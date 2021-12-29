use std::collections::VecDeque;
#[cfg(feature = "music_library")]
use std::path::PathBuf;

use super::SyntaxError;
use crate::tokens::MpsToken;
use super::MpsTypePrimitive;

pub fn assert_token<T, F: FnOnce(MpsToken) -> Option<T>>(
    caster: F,
    token: MpsToken,
    tokens: &mut VecDeque<MpsToken>,
) -> Result<T, SyntaxError> {
    if let Some(out) = caster(tokens.pop_front().unwrap()) {
        Ok(out)
    } else {
        Err(SyntaxError {
            line: 0,
            token: token,
        })
    }
}

pub fn assert_token_raw(
    token: MpsToken,
    tokens: &mut VecDeque<MpsToken>,
) -> Result<MpsToken, SyntaxError> {
    let result = tokens.pop_front().unwrap();
    if std::mem::discriminant(&token) == std::mem::discriminant(&result) {
        Ok(result)
    } else {
        Err(SyntaxError {
            line: 0,
            token: token,
        })
    }
}

pub fn check_token_raw(
    token: MpsToken,
    token_target: &MpsToken,
) -> bool {
    std::mem::discriminant(&token) == std::mem::discriminant(token_target)
}

pub fn assert_name(name: &str, tokens: &mut VecDeque<MpsToken>) -> Result<String, SyntaxError> {
    match tokens.pop_front().unwrap() {
        MpsToken::Name(n) => {
            if n == name {
                Ok(n)
            } else {
                Err(
                    SyntaxError {
                        line: 0,
                        token: MpsToken::Name(name.to_owned()),
                    }
                )
            }
        },
        _token => Err(SyntaxError {
            line: 0,
            token: MpsToken::Name(name.to_owned()),
        })
    }
}

pub fn check_name(name: &str, token: &MpsToken) -> bool {
    match token {
        MpsToken::Name(n) => n == name,
        _ => false
    }
}

pub fn check_is_type(token: &MpsToken) -> bool {
    match token {
        MpsToken::Literal(_) => true,
        MpsToken::Name(s) =>
            s.parse::<f64>().is_ok()
            || s.parse::<i64>().is_ok()
            || s.parse::<u64>().is_ok()
            || s == "false"
            || s == "true",
        _ => false
    }
}

pub fn assert_type(tokens: &mut VecDeque<MpsToken>) -> Result<MpsTypePrimitive, SyntaxError> {
    match tokens.pop_front().unwrap() {
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
                })
            }
        },
        _token => Err(SyntaxError {
            line: 0,
            token: MpsToken::Name("Float | UInt | Int | Bool | \"String\"".into()),
        })
    }
}

pub fn music_folder() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("./"))
        .join("Music")
}

