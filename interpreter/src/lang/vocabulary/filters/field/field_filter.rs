use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use super::super::utility::{assert_comparison_operator, comparison_op};
use super::{FieldFilterFactory, FieldFilterPredicate};
use crate::lang::utility::{assert_token, assert_type, check_is_type};
use crate::lang::LanguageDictionary;
use crate::lang::TypePrimitive;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;
use crate::Item;

#[derive(Debug, Clone)]
pub(super) enum VariableOrValue {
    Variable(String),
    Value(TypePrimitive),
}

#[derive(Debug, Clone)]
pub struct FieldFilter {
    pub(super) field_name: String,
    pub(super) field_errors: FieldFilterErrorHandling,
    pub(super) comparison_errors: FieldFilterErrorHandling,
    pub(super) val: VariableOrValue,
    pub(super) comparison: [i8; 2],
}

#[derive(Debug, Clone)]
pub enum FieldFilterErrorHandling {
    Error,   // return error
    Ignore,  // return Ok(false) when error encountered
    Include, // return Ok(true) when error encountered
}

impl Display for FieldFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let comp_op = comparison_op(&self.comparison);
        match &self.val {
            VariableOrValue::Variable(name) => {
                write!(f, ".{} {} {}", self.field_name, comp_op, name)
            }
            VariableOrValue::Value(t) => write!(f, "{} {} {}", self.field_name, comp_op, t),
        }
    }
}

impl FieldFilterPredicate for FieldFilter {
    fn matches(&mut self, music_item_lut: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg> {
        let variable = match &self.val {
            VariableOrValue::Variable(name) => match ctx.variables.get(name)? {
                Type::Primitive(t) => Ok(t),
                _ => Err(RuntimeMsg(format!("Variable {} is not comparable", name))),
            },
            VariableOrValue::Value(val) => Ok(val),
        }?;
        if let Some(field) = music_item_lut.field(&self.field_name) {
            let compare_res = field.compare(variable);
            if let Err(e) = compare_res {
                match self.comparison_errors {
                    FieldFilterErrorHandling::Error => Err(RuntimeMsg(e)),
                    FieldFilterErrorHandling::Ignore => Ok(false),
                    FieldFilterErrorHandling::Include => Ok(true),
                }
            } else {
                let compare = compare_res.unwrap();
                let mut is_match = false;
                for comparator in self.comparison {
                    if comparator == compare {
                        is_match = true;
                        break;
                    }
                }
                Ok(is_match)
            }
        } else {
            match self.field_errors {
                FieldFilterErrorHandling::Error => Err(RuntimeMsg(format!(
                    "Field {} does not exist on item",
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

pub struct FieldFilterComparisonFactory;

impl FieldFilterFactory<FieldFilter> for FieldFilterComparisonFactory {
    fn is_filter(&self, tokens: &[Token]) -> bool {
        let tokens_len = tokens.len();
        (tokens_len >= 1
            // .field > variable OR .field < variable
            && (tokens[0].is_open_angle_bracket() || tokens[0].is_close_angle_bracket()))
            || (tokens_len >= 2 // .field >= variable OR .field <= variable OR .field != variable OR .field == variable
            && (tokens[0].is_open_angle_bracket() || tokens[0].is_close_angle_bracket() || tokens[0].is_equals() || tokens[0].is_exclamation())
            && tokens[1].is_equals()
            && !(tokens_len > 2 && tokens[2].is_equals())
            )
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        field: String,
        _dict: &LanguageDictionary,
    ) -> Result<FieldFilter, SyntaxError> {
        let compare_operator = assert_comparison_operator(tokens)?;
        if check_is_type(&tokens[0]) {
            let value = VariableOrValue::Value(assert_type(tokens)?);
            //assert_empty(tokens)?;
            Ok(FieldFilter {
                field_name: field,
                field_errors: FieldFilterErrorHandling::Error,
                comparison_errors: FieldFilterErrorHandling::Error,
                val: value,
                comparison: compare_operator,
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
            Ok(FieldFilter {
                field_name: field,
                field_errors: FieldFilterErrorHandling::Error,
                comparison_errors: FieldFilterErrorHandling::Error,
                val: variable,
                comparison: compare_operator,
            })
        }
    }
}
