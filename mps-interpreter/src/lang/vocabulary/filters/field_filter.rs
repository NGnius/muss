use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use super::utility::{assert_comparison_operator, comparison_op};
use crate::lang::utility::{assert_token, assert_type, check_is_type, assert_empty};
use crate::lang::MpsLanguageDictionary;
use crate::lang::MpsTypePrimitive;
use crate::lang::{MpsFilterFactory, MpsFilterPredicate, MpsFilterStatementFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

#[derive(Debug, Clone)]
pub(super) enum VariableOrValue {
    Variable(String),
    Value(MpsTypePrimitive),
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
                write!(f, "{} {} {}", self.field_name, comp_op, name)
            }
            VariableOrValue::Value(t) => write!(f, "{} {} {}", self.field_name, comp_op, t),
        }
    }
}

impl MpsFilterPredicate for FieldFilter {
    fn matches(
        &mut self,
        music_item_lut: &MpsItem,
        ctx: &mut MpsContext,
    ) -> Result<bool, RuntimeMsg> {
        let variable = match &self.val {
            VariableOrValue::Variable(name) => match ctx.variables.get(name)? {
                MpsType::Primitive(t) => Ok(t),
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

pub struct FieldFilterFactory;

impl MpsFilterFactory<FieldFilter> for FieldFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        let tokens_len = tokens.len();
        (tokens_len >= 3
            // field > variable OR field < variable
            && tokens[0].is_name()
            && (tokens[1].is_open_angle_bracket() || tokens[1].is_close_angle_bracket())
            && (tokens[2].is_name() || check_is_type(tokens[2])))
            || (tokens_len >= 4 // field >= variable OR field <= variable OR field != variable
            && tokens[0].is_name()
            && (tokens[1].is_open_angle_bracket() || tokens[1].is_close_angle_bracket() || tokens[1].is_equals() || tokens[1].is_exclamation())
            && tokens[2].is_equals()
            && (tokens[3].is_name() || check_is_type(tokens[3])))
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<FieldFilter, SyntaxError> {
        let field = assert_token(
            |t| match t {
                MpsToken::Name(n) => Some(n),
                _ => None,
            },
            MpsToken::Name("field_name".into()),
            tokens,
        )?;
        let compare_operator = assert_comparison_operator(tokens)?;
        if check_is_type(&tokens[0]) {
            let value = VariableOrValue::Value(assert_type(tokens)?);
            assert_empty(tokens)?;
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
                    MpsToken::Name(n) => Some(n),
                    _ => None,
                },
                MpsToken::Name("variable_name".into()),
                tokens,
            )?);
            assert_empty(tokens)?;
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

pub type FieldFilterStatementFactory = MpsFilterStatementFactory<FieldFilter, FieldFilterFactory>;

#[inline(always)]
pub fn field_filter() -> FieldFilterStatementFactory {
    FieldFilterStatementFactory::new(FieldFilterFactory)
}
