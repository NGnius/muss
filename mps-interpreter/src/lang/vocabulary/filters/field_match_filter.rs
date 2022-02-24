use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use regex::Regex;

use super::field_filter::{FieldFilterErrorHandling, VariableOrValue};
use crate::lang::utility::{assert_name, assert_token, assert_token_raw, check_name, assert_empty};
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
        let pattern = if let Some((_, regex_c)) = &self.regex_cache {
            regex_c
        } else {
            let regex_c = Regex::new(variable)
                .map_err(|e| RuntimeMsg(format!("Regex compile error: {}", e)))?;
            self.regex_cache = Some((variable.clone(), regex_c));
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
        Ok(())
    }
}

pub struct FieldRegexFilterFactory;

impl MpsFilterFactory<FieldRegexFilter> for FieldRegexFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        let tokens_len = tokens.len();
        (tokens_len >= 3 // field like variable
            && tokens[0].is_name()
            && check_name("matches", tokens[1])
            && (tokens[2].is_name() || tokens[2].is_literal()))
            || (tokens_len >= 4 // field? like variable OR field! like variable
            && tokens[0].is_name()
            && (tokens[1].is_interrogation() || tokens[1].is_exclamation())
            && check_name("matches", tokens[2])
            && (tokens[3].is_name() || tokens[3].is_literal()))
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
            let regex_c = Regex::new(&literal).map_err(|_| SyntaxError {
                line: 0,
                token: MpsToken::Literal("[valid regex]".to_string()),
                got: Some(MpsToken::Literal(literal.clone())),
            })?;
            let compiled_cache = (literal.clone(), regex_c);
            let value = VariableOrValue::Value(MpsTypePrimitive::String(literal));
            assert_empty(tokens)?;
            Ok(FieldRegexFilter {
                field_name: field,
                field_errors: error_handling,
                val: value,
                regex_cache: Some(compiled_cache),
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
            assert_empty(tokens)?;
            Ok(FieldRegexFilter {
                field_name: field,
                field_errors: FieldFilterErrorHandling::Error,
                val: variable,
                regex_cache: None,
            })
        }
    }
}

pub type FieldRegexFilterStatementFactory =
    MpsFilterStatementFactory<FieldRegexFilter, FieldRegexFilterFactory>;

#[inline(always)]
pub fn field_re_filter() -> FieldRegexFilterStatementFactory {
    FieldRegexFilterStatementFactory::new(FieldRegexFilterFactory)
}
