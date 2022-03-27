use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use regex::{Regex, RegexBuilder};

use super::field_filter::{FieldFilterErrorHandling, VariableOrValue};
use crate::lang::utility::{assert_name, assert_token, assert_token_raw, check_name};
use crate::lang::MpsLanguageDictionary;
use crate::lang::MpsTypePrimitive;
use crate::lang::{MpsFilterFactory, MpsFilterPredicate, MpsFilterStatementFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

#[derive(Debug, Clone)]
pub struct FieldRegexFilter {
    field_name: String,
    field_errors: FieldFilterErrorHandling,
    val: VariableOrValue,
    regex_cache: Option<(String, Regex)>,
    regex_options: u8,
}

impl Display for FieldRegexFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match &self.val {
            VariableOrValue::Variable(name) => write!(f, "{} matches {}", self.field_name, name),
            VariableOrValue::Value(t) => write!(f, "{} matches {}", self.field_name, t),
        }
    }
}

impl MpsFilterPredicate for FieldRegexFilter {
    fn matches(
        &mut self,
        music_item_lut: &MpsItem,
        ctx: &mut MpsContext,
    ) -> Result<bool, RuntimeMsg> {
        let variable = match &self.val {
            VariableOrValue::Variable(name) => match ctx.variables.get(name)? {
                MpsType::Primitive(MpsTypePrimitive::String(s)) => Ok(s),
                _ => Err(RuntimeMsg(format!("Variable {} is not comparable", name))),
            },
            VariableOrValue::Value(MpsTypePrimitive::String(s)) => Ok(s),
            // non-string values will be stopped at parse-time, so this should never occur
            _ => Err(RuntimeMsg("Value is not type String".to_string())),
        }?;
        let pattern = if let Some((val, regex_c)) = &self.regex_cache {
            if val == variable {
                regex_c
            } else {
                // only rebuild regex when variable's value changes
                let regex_c = build_regex(variable, self.regex_options)
                    .map_err(|e| RuntimeMsg(format!("Regex compile error: {}", e)))?;
                self.regex_cache = Some((variable.to_owned(), regex_c));
                &self.regex_cache.as_ref().unwrap().1
            }
        } else {
            // build empty cache
            let regex_c = build_regex(variable, self.regex_options)
                .map_err(|e| RuntimeMsg(format!("Regex compile error: {}", e)))?;
            self.regex_cache = Some((variable.to_owned(), regex_c));
            &self.regex_cache.as_ref().unwrap().1
        };
        if let Some(field) = music_item_lut.field(&self.field_name) {
            let field_str = field.as_str();
            Ok(pattern.is_match(&field_str))
        } else {
            match self.field_errors {
                FieldFilterErrorHandling::Error => Err(RuntimeMsg(format!(
                    "Field {} does not exist",
                    &self.field_name
                ))),
                FieldFilterErrorHandling::Ignore => Ok(false),
                FieldFilterErrorHandling::Include => Ok(true),
            }
        }
    }

    fn is_complete(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        //self.regex_cache = None;
        Ok(())
    }
}

pub struct FieldRegexFilterFactory;

impl MpsFilterFactory<FieldRegexFilter> for FieldRegexFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        let tokens_len = tokens.len();
        (tokens_len >= 2 // field like variable
            && tokens[0].is_name()
            && check_name("matches", tokens[1]))
            || (tokens_len >= 3 // field? like variable OR field! like variable
            && tokens[0].is_name()
            && (tokens[1].is_interrogation() || tokens[1].is_exclamation())
            && check_name("matches", tokens[2]))
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<FieldRegexFilter, SyntaxError> {
        let field = assert_token(
            |t| match t {
                MpsToken::Name(n) => Some(n),
                _ => None,
            },
            MpsToken::Name("field_name".into()),
            tokens,
        )?;
        let error_handling = if tokens[0].is_interrogation() {
            assert_token_raw(MpsToken::Interrogation, tokens)?;
            FieldFilterErrorHandling::Ignore
        } else if tokens[0].is_exclamation() {
            assert_token_raw(MpsToken::Exclamation, tokens)?;
            FieldFilterErrorHandling::Include
        } else {
            FieldFilterErrorHandling::Error
        };
        assert_name("matches", tokens)?;
        if tokens[0].is_literal() {
            let literal = assert_token(
                |t| match t {
                    MpsToken::Literal(n) => Some(n),
                    _ => None,
                },
                MpsToken::Literal("regex_string".into()),
                tokens,
            )?;
            let re_flags = regex_flags(tokens)?;
            let regex_c = build_regex(&literal, re_flags).map_err(|_| SyntaxError {
                line: 0,
                token: MpsToken::Literal("[valid regex]".to_string()),
                got: Some(MpsToken::Literal(literal.clone())),
            })?;
            let compiled_cache = (literal.clone(), regex_c);
            let value = VariableOrValue::Value(MpsTypePrimitive::String(literal));
            //assert_empty(tokens)?;
            Ok(FieldRegexFilter {
                field_name: field,
                field_errors: error_handling,
                val: value,
                regex_cache: Some(compiled_cache),
                regex_options: re_flags,
            })
        } else {
            let variable = VariableOrValue::Variable(assert_token(
                |t| match t {
                    MpsToken::Name(n) => Some(n),
                    _ => None,
                },
                MpsToken::Name("variable_name".into()),
                tokens,
            )?);
            //assert_empty(tokens)?;
            Ok(FieldRegexFilter {
                field_name: field,
                field_errors: FieldFilterErrorHandling::Error,
                val: variable,
                regex_cache: None,
                regex_options: regex_flags(tokens)?,
            })
        }
    }
}

#[inline]
fn regex_flags(tokens: &mut VecDeque<MpsToken>) -> Result<u8, SyntaxError> {
    // syntax: , "flags"
    let mut result = 0_u8;
    if tokens.is_empty() {
        Ok(result)
    } else {
        assert_token_raw(MpsToken::Comma, tokens)?;
        let flags = assert_token(
            |t| match t {
                MpsToken::Literal(s) => Some(s),
                _ => None,
            },
            MpsToken::Literal("[one or more of imsUux]".into()),
            tokens,
        )?;
        // build flag byte
        for c in flags.chars() {
            match c {
                'i' => result |= 1 << 0,
                'm' => result |= 1 << 1,
                's' => result |= 1 << 2,
                'U' => result |= 1 << 3,
                'u' => result |= 1 << 4,
                'x' => result |= 1 << 5,
                c => {
                    return Err(SyntaxError {
                        line: 0,
                        token: MpsToken::Literal("[one or more of imsUux]".to_string()),
                        got: Some(MpsToken::Literal(format!("{}", c))),
                    })
                }
            }
        }
        Ok(result)
    }
}

#[inline]
fn build_regex(pattern: &str, flags: u8) -> Result<Regex, regex::Error> {
    RegexBuilder::new(pattern)
        .case_insensitive((flags & (1 << 0)) != 0)
        .multi_line((flags & (1 << 1)) != 0)
        .dot_matches_new_line((flags & (1 << 2)) != 0)
        .swap_greed((flags & (1 << 3)) != 0)
        .unicode((flags & (1 << 4)) != 0)
        .ignore_whitespace((flags & (1 << 5)) != 0)
        .build()
}

pub type FieldRegexFilterStatementFactory =
    MpsFilterStatementFactory<FieldRegexFilter, FieldRegexFilterFactory>;

#[inline(always)]
pub fn field_re_filter() -> FieldRegexFilterStatementFactory {
    FieldRegexFilterStatementFactory::new(FieldRegexFilterFactory)
}
