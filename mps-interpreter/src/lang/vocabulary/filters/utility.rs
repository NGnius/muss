use std::collections::HashMap;
use std::collections::VecDeque;

use crate::lang::utility::assert_token_raw;
use crate::lang::MpsTypePrimitive;
use crate::lang::SyntaxError;
use crate::tokens::MpsToken;
use crate::MpsMusicItem;

pub fn item_to_primitive_lut(item: MpsMusicItem) -> HashMap<String, MpsTypePrimitive> {
    let mut result = HashMap::new();
    result.insert("title".into(), MpsTypePrimitive::String(item.title));
    result.insert(
        "artist".into(),
        MpsTypePrimitive::String(item.artist.unwrap_or("".to_owned())),
    );
    result.insert(
        "album".into(),
        MpsTypePrimitive::String(item.album.unwrap_or("".to_owned())),
    );
    result.insert("filename".into(), MpsTypePrimitive::String(item.filename));
    result.insert(
        "genre".into(),
        MpsTypePrimitive::String(item.genre.unwrap_or("".to_owned())),
    );
    result.insert(
        "track".into(),
        MpsTypePrimitive::UInt(item.track.unwrap_or(0)),
    );
    result.insert(
        "year".into(),
        MpsTypePrimitive::UInt(item.year.unwrap_or(0)),
    );
    result
}

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
        _ => Err(SyntaxError {
            line: 0,
            token: MpsToken::Equals, // TODO this can be < > or =
            got: Some(token1),
        }),
    }
}
