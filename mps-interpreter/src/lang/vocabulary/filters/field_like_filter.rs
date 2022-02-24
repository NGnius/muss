use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

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
pub struct FieldLikeFilter {
    field_name: String,
    field_errors: FieldFilterErrorHandling,
    val: VariableOrValue,
}

impl Display for FieldLikeFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match &self.val {
            VariableOrValue::Variable(name) => write!(f, "{} like {}", self.field_name, name),
            VariableOrValue::Value(t) => write!(f, "{} like {}", self.field_name, t),
        }
    }
}

impl MpsFilterPredicate for FieldLikeFilter {
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
        if let Some(field) = music_item_lut.field(&self.field_name) {
            let field_str = field.as_str().to_lowercase();
            Ok(field_str.contains(&variable.to_lowercase()))
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

pub struct FieldLikeFilterFactory;

impl MpsFilterFactory<FieldLikeFilter> for FieldLikeFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        let tokens_len = tokens.len();
        (tokens_len >= 2 // field like variable
            && tokens[0].is_name()
            && check_name("like", tokens[1]))
            || (tokens_len >= 3 // field? like variable OR field! like variable
            && tokens[0].is_name()
            && (tokens[1].is_interrogation() || tokens[1].is_exclamation())
            && check_name("like", tokens[2]))
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<FieldLikeFilter, SyntaxError> {
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
        assert_name("like", tokens)?;
        if tokens[0].is_literal() {
            let literal = assert_token(
                |t| match t {
                    MpsToken::Literal(n) => Some(n),
                    _ => None,
                },
                MpsToken::Literal("like_string".into()),
                tokens,
            )?;
            let value = VariableOrValue::Value(MpsTypePrimitive::String(literal));
            //assert_empty(tokens)?;
            Ok(FieldLikeFilter {
                field_name: field,
                field_errors: error_handling,
                val: value,
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
            Ok(FieldLikeFilter {
                field_name: field,
                field_errors: FieldFilterErrorHandling::Error,
                val: variable,
            })
        }
    }
}

pub type FieldLikeFilterStatementFactory =
    MpsFilterStatementFactory<FieldLikeFilter, FieldLikeFilterFactory>;

#[inline(always)]
pub fn field_like_filter() -> FieldLikeFilterStatementFactory {
    FieldLikeFilterStatementFactory::new(FieldLikeFilterFactory)
}
