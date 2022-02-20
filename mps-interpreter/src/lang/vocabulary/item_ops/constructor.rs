use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token_raw, assert_token_raw_back, check_name, assert_name, assert_token};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;

#[derive(Debug)]
struct FieldAssignment {
    name: String,
    value: Box<dyn MpsItemOp>,
}

#[derive(Debug)]
pub struct ConstructorItemOp {
    fields: Vec<FieldAssignment>,
}

impl Deref for ConstructorItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for ConstructorItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Item(")?;
        if self.fields.len() > 1 {
            write!(f, "\n")?;
            for i in 0..self.fields.len()-1 {
                let field = &self.fields[i];
                write!(f, "{}: {}, ", field.name, field.value)?;
            }
            let field = &self.fields[self.fields.len()-1];
            write!(f, "{}: {}", field.name, field.value)?;
        } else if !self.fields.is_empty() {
            let field = &self.fields[0];
            write!(f, "{}: {}", field.name, field.value)?;
        }
        write!(f, ")")
    }
}

impl MpsItemOp for ConstructorItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let mut result = MpsItem::new();
        for field in &self.fields {
            let value = field.value.execute(context)?;
            if let MpsType::Primitive(value) = value {
                result.set_field(&field.name, value);
            } else {
                return Err(RuntimeMsg(format!("Cannot assign non-primitive {} to Item field `{}`", value, &field.name)));
            }
        }
        Ok(MpsType::Item(result))
    }
}

pub struct ConstructorItemOpFactory;

impl MpsItemOpFactory<ConstructorItemOp> for ConstructorItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() > 2
        && check_name("Item", &tokens[0])
        && tokens[1].is_open_bracket()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<ConstructorItemOp, SyntaxError> {
        assert_name("Item", tokens)?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        assert_token_raw_back(MpsToken::CloseBracket, tokens)?;
        let mut field_descriptors = Vec::new();
        while !tokens.is_empty() {
            let field_name = assert_token(|t| match t {
                MpsToken::Name(n) => Some(n),
                _ => None,
            }, MpsToken::Name("field_name".into()), tokens)?;
            assert_token_raw(MpsToken::Equals, tokens)?;
            let field_val;
            if let Some(comma_pos) = find_next_comma(tokens) {
                let end_tokens = tokens.split_off(comma_pos);
                field_val = factory.try_build_item_statement(tokens, dict)?;
                tokens.extend(end_tokens);
                assert_token_raw(MpsToken::Comma, tokens)?;
            } else {
                field_val = factory.try_build_item_statement(tokens, dict)?;
            }
            field_descriptors.push(
                FieldAssignment {
                    name: field_name,
                    value: field_val,
                }
            );
        }
        Ok(ConstructorItemOp {
            fields: field_descriptors,
        })
    }
}

fn find_next_comma(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    let mut curly_depth = 0;
    for i in 0..tokens.len() {
        let token = &tokens[i];
        if token.is_comma() && bracket_depth == 0 && curly_depth == 0 {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        } else if token.is_open_curly() {
            curly_depth += 1;
        } else if token.is_close_curly() && curly_depth != 0 {
            curly_depth -= 1;
        }
    }
    None
}
