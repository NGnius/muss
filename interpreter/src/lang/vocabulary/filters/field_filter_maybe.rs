use std::collections::VecDeque;

use super::utility::assert_comparison_operator;
use super::{field_filter::VariableOrValue, FieldFilter, FieldFilterErrorHandling};
use crate::lang::utility::{assert_token, assert_token_raw, assert_type, check_is_type};
use crate::lang::LanguageDictionary;
use crate::lang::SyntaxError;
use crate::lang::{FilterFactory, FilterStatementFactory};
use crate::tokens::Token;

pub struct FieldFilterMaybeFactory;

impl FilterFactory<FieldFilter> for FieldFilterMaybeFactory {
    fn is_filter(&self, tokens: &VecDeque<&Token>) -> bool {
        let tokens_len = tokens.len();
        (tokens_len >= 3 // field > variable OR field < variable
            && tokens[0].is_name()
            && (tokens[1].is_interrogation() || tokens[1].is_exclamation())
            && (tokens[2].is_open_angle_bracket() || tokens[2].is_close_angle_bracket()))
            || (tokens_len >= 4 // field >= variable OR field <= variable OR field != variable
            && tokens[0].is_name()
            && (tokens[1].is_interrogation() || tokens[1].is_exclamation())
            && (tokens[2].is_open_angle_bracket() || tokens[2].is_close_angle_bracket() || tokens[2].is_equals() || tokens[2].is_exclamation())
            && tokens[3].is_equals())
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<FieldFilter, SyntaxError> {
        let field = assert_token(
            |t| match t {
                Token::Name(n) => Some(n),
                _ => None,
            },
            Token::Name("field_name".into()),
            tokens,
        )?;
        let error_f;
        let error_c;
        if tokens[0].is_interrogation() {
            error_f = FieldFilterErrorHandling::Ignore;
            error_c = FieldFilterErrorHandling::Ignore;
            assert_token_raw(Token::Interrogation, tokens)?;
        } else {
            error_f = FieldFilterErrorHandling::Include;
            error_c = FieldFilterErrorHandling::Include;
            assert_token_raw(Token::Exclamation, tokens)?;
        }
        let compare_operator = assert_comparison_operator(tokens)?;
        if check_is_type(&tokens[0]) {
            let value = VariableOrValue::Value(assert_type(tokens)?);
            //assert_empty(tokens)?;
            Ok(FieldFilter {
                field_name: field,
                field_errors: error_f,
                comparison_errors: error_c,
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
                field_errors: error_f,
                comparison_errors: error_c,
                val: variable,
                comparison: compare_operator,
            })
        }
    }
}

pub type FieldFilterMaybeStatementFactory =
    FilterStatementFactory<FieldFilter, FieldFilterMaybeFactory>;

#[inline(always)]
pub fn field_filter_maybe() -> FieldFilterMaybeStatementFactory {
    FieldFilterMaybeStatementFactory::new(FieldFilterMaybeFactory)
}
