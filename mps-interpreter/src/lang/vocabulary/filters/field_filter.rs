use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::MpsContext;
use crate::MpsMusicItem;
use crate::tokens::MpsToken;
use crate::lang::{MpsFilterPredicate, MpsFilterFactory, MpsFilterStatementFactory};
use crate::lang::{SyntaxError, RuntimeError};
use crate::lang::MpsLanguageDictionary;
use crate::lang::MpsTypePrimitive;
use crate::lang::utility::{assert_token, assert_type, check_is_type};
use super::utility::{item_to_primitive_lut, assert_comparison_operator};
use crate::processing::OpGetter;
use crate::processing::general::MpsType;

#[derive(Debug, Clone)]
enum VariableOrValue {
    Variable(String),
    Value(MpsTypePrimitive),
}

#[derive(Debug, Clone)]
pub struct FieldFilter {
    field_name: String,
    val: VariableOrValue,
    comparison: [i8; 2]
}

impl Display for FieldFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        // TODO display other comparison operators correctly
        match &self.val {
            VariableOrValue::Variable(name) => write!(f, "{} == {}", self.field_name, name),
            VariableOrValue::Value(t) => write!(f, "{} == {}", self.field_name, t),
        }

    }
}

impl MpsFilterPredicate for FieldFilter {
    fn matches(&mut self, item: &MpsMusicItem, ctx: &mut MpsContext, op: &mut OpGetter) -> Result<bool, RuntimeError> {
        let music_item_lut = item_to_primitive_lut(item.to_owned());
        let variable = match &self.val {
            VariableOrValue::Variable(name) => match ctx.variables.get(&name, op)? {
                MpsType::Primitive(t) => Ok(t),
                _ => Err(RuntimeError {
                    line: 0,
                    op: op(),
                    msg: format!("Variable {} is not comparable", name),
                })
            },
            VariableOrValue::Value(val) => Ok(val)
        }?;
        if let Some(field) = music_item_lut.get(&self.field_name) {
            let compare = field.compare(variable)
                .map_err(|e| RuntimeError {
                    line: 0,
                    op: op(),
                    msg: e,
                })?;
            let mut is_match = false;
            for comparator in self.comparison {
                if comparator == compare {
                    is_match = true;
                    break;
                }
            }
            Ok(is_match)
        } else {
            Err(RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Field {} does not exist", &self.field_name),
            })
        }

    }
}

pub struct FieldFilterFactory;

impl MpsFilterFactory<FieldFilter> for FieldFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        let tokens_len = tokens.len();
        (tokens_len == 3 // field > variable OR field < variable
            && tokens[0].is_name()
            && (tokens[1].is_open_angle_bracket() || tokens[1].is_close_angle_bracket())
            && (tokens[2].is_name() || check_is_type(&tokens[2]))
        )
        || (tokens_len == 4 // field >= variable OR field <= variable
            && tokens[0].is_name()
            && (tokens[1].is_open_angle_bracket() || tokens[1].is_close_angle_bracket() || tokens[1].is_equals())
            && tokens[2].is_equals()
            && (tokens[3].is_name() || check_is_type(&tokens[3]))
        )
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<FieldFilter, SyntaxError> {
        let field = assert_token(|t| match t {
            MpsToken::Name(n) => Some(n),
            _ => None
        }, MpsToken::Name("field_name".into()), tokens)?;
        let compare_operator = assert_comparison_operator(tokens)?;
        if check_is_type(&tokens[0]) {
            let value = VariableOrValue::Value(assert_type(tokens)?);
            Ok(FieldFilter{
                field_name: field,
                val: value,
                comparison: compare_operator,
            })
        } else {
            let variable = VariableOrValue::Variable(
                assert_token(|t| match t {
                    MpsToken::Name(n) => Some(n),
                    _ => None
                }, MpsToken::Name("variable_name".into()), tokens)?
            );
            Ok(FieldFilter{
                field_name: field,
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
