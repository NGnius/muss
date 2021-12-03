use std::collections::VecDeque;
#[cfg(feature = "music_library")]
use std::path::PathBuf;

use crate::tokens::MpsToken;
use super::SyntaxError;

pub fn assert_token<T, F: FnOnce(MpsToken) -> Option<T>>(
    caster: F,
    token: MpsToken,
    tokens: &mut VecDeque<MpsToken>
) -> Result<T, SyntaxError> {
    if let Some(out) = caster(tokens.pop_front().unwrap()) {
        Ok(out)
    } else {
        Err(SyntaxError{
            line: 0,
            token: token,
        })
    }
}

pub fn assert_token_raw(
    token: MpsToken,
    tokens: &mut VecDeque<MpsToken>
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

#[cfg(feature = "music_library")]
pub fn music_folder() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("./")).join("Music")
}
