use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use super::field_filter::{FieldFilterErrorHandling, VariableOrValue};
use crate::lang::utility::{assert_token, assert_token_raw, check_name};
use crate::lang::LanguageDictionary;
use crate::lang::TypePrimitive;
use super::{FieldFilterFactory, FieldFilterPredicate};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;
use crate::Item;

#[derive(Debug, Clone)]
pub struct FieldLikeFilter {
    field_name: String,
    field_errors: FieldFilterErrorHandling,
    val: VariableOrValue,
    negate: bool,
}

impl FieldLikeFilter {
    fn sanitise_string(s: &str) -> String {
        super::super::utility::sanitise_string(s)
    }
}

impl Display for FieldLikeFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match &self.val {
            VariableOrValue::Variable(name) => write!(f, ".{} like {}", self.field_name, name),
            VariableOrValue::Value(t) => write!(f, ".{} like {}", self.field_name, t),
        }
    }
}

impl FieldFilterPredicate for FieldLikeFilter {
    fn matches(&mut self, music_item_lut: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg> {
        let variable = match &self.val {
            VariableOrValue::Variable(name) => match ctx.variables.get(name)? {
                Type::Primitive(TypePrimitive::String(s)) => Ok(s),
                _ => Err(RuntimeMsg(format!("Variable {} is not comparable", name))),
            },
            VariableOrValue::Value(TypePrimitive::String(s)) => Ok(s),
            // non-string values will be stopped at parse-time, so this should never occur
            _ => Err(RuntimeMsg("Value is not type String".to_string())),
        }?;
        if let Some(field) = music_item_lut.field(&self.field_name) {
            let field_str = Self::sanitise_string(&field.as_str());
            let var_str = Self::sanitise_string(variable);
            let matches = field_str.contains(&var_str);
            if self.negate {
                Ok(!matches)
            } else {
                Ok(matches)
            }
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

    fn box_clone(&self) -> Box<dyn FieldFilterPredicate + 'static> {
        Box::new(self.clone())
    }
}

pub struct FieldLikeFilterFactory;

impl FieldFilterFactory<FieldLikeFilter> for FieldLikeFilterFactory {
    fn is_filter(&self, tokens: &[Token]) -> bool {
        let tokens_len = tokens.len();
        (tokens_len >= 1 // field like variable
            && (check_name("like", &tokens[0]) || check_name("unlike", &tokens[0])))
            || (tokens_len >= 2 // field? like variable OR field! like variable
            && (tokens[0].is_interrogation() || tokens[0].is_exclamation())
            && (check_name("like", &tokens[1]) || check_name("unlike", &tokens[1])))
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        field: String,
        _dict: &LanguageDictionary,
    ) -> Result<FieldLikeFilter, SyntaxError> {
        let error_handling = if tokens[0].is_interrogation() {
            assert_token_raw(Token::Interrogation, tokens)?;
            FieldFilterErrorHandling::Ignore
        } else if tokens[0].is_exclamation() {
            assert_token_raw(Token::Exclamation, tokens)?;
            FieldFilterErrorHandling::Include
        } else {
            FieldFilterErrorHandling::Error
        };
        let name = assert_token(
            |t| match t {
                Token::Name(s) => match &s as _ {
                    "unlike" | "like" => Some(s),
                    _ => None,
                },
                _ => None,
            },
            Token::Literal("like|unlike".into()),
            tokens,
        )?;
        let is_negated = name == "unlike";
        //assert_name("like", tokens)?;
        if tokens[0].is_literal() {
            let literal = assert_token(
                |t| match t {
                    Token::Literal(n) => Some(n),
                    _ => None,
                },
                Token::Literal("like_string".into()),
                tokens,
            )?;
            let value = VariableOrValue::Value(TypePrimitive::String(literal));
            //assert_empty(tokens)?;
            Ok(FieldLikeFilter {
                field_name: field,
                field_errors: error_handling,
                val: value,
                negate: is_negated,
            })
        } else {
            let variable = VariableOrValue::Variable(assert_token(
                |t| match t {
                    Token::Name(n) => Some(n),
                    _ => None,
                },
                Token::Name("variable_name".into()),
                tokens,
            )?);
            //assert_empty(tokens)?;
            Ok(FieldLikeFilter {
                field_name: field,
                field_errors: FieldFilterErrorHandling::Error,
                val: variable,
                negate: is_negated,
            })
        }
    }
}
