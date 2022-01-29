use std::collections::VecDeque;

use crate::lang::utility::{assert_token_raw, check_token_raw};
use crate::lang::SyntaxError;
use crate::tokens::MpsToken;

/// Convenient parser for repeated patterns of tokens
pub struct RepeatedTokens<
    X: 'static,
    F1: FnMut(&mut VecDeque<MpsToken>) -> Result<Option<X>, SyntaxError>,
    F2: FnMut(&mut VecDeque<MpsToken>) -> Result<bool, SyntaxError>,
> {
    pattern_ingest: F1,
    separator_ingest: F2,
}

impl<
        X: 'static,
        F1: FnMut(&mut VecDeque<MpsToken>) -> Result<Option<X>, SyntaxError>,
        F2: FnMut(&mut VecDeque<MpsToken>) -> Result<bool, SyntaxError>,
    > RepeatedTokens<X, F1, F2>
{
    pub fn ingest_all(&mut self, tokens: &mut VecDeque<MpsToken>) -> Result<Vec<X>, SyntaxError> {
        let mut result = Vec::<X>::new();
        match (self.pattern_ingest)(tokens)? {
            Some(x) => result.push(x),
            None => return Ok(result),
        }
        while (self.separator_ingest)(tokens)? {
            match (self.pattern_ingest)(tokens)? {
                Some(x) => result.push(x),
                None => break,
            }
        }
        Ok(result)
    }
}

pub fn repeated_tokens<X, F1: FnMut(&mut VecDeque<MpsToken>) -> Result<Option<X>, SyntaxError>>(
    ingestor: F1,
    separator: MpsToken,
) -> RepeatedTokens<X, F1, impl FnMut(&mut VecDeque<MpsToken>) -> Result<bool, SyntaxError>> {
    RepeatedTokens {
        pattern_ingest: ingestor,
        separator_ingest: move |tokens| {
            if !tokens.is_empty() && check_token_raw(separator.clone(), &tokens[0]) {
                assert_token_raw(separator.clone(), tokens)?;
                Ok(true)
            } else {
                Ok(false)
            }
        },
        //parsed: Vec::new(),
    }
}
