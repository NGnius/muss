use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{
    assert_name, assert_token, assert_token_raw, check_name,
};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;
use crate::Item;

#[derive(Debug)]
struct FieldAssignment {
    name: String,
    value: Box<dyn ItemOp>,
}

#[derive(Debug)]
pub struct ConstructorItemOp {
    fields: Vec<FieldAssignment>,
}

impl Deref for ConstructorItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for ConstructorItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Item(")?;
        if self.fields.len() > 1 {
            writeln!(f)?;
            for i in 0..self.fields.len() - 1 {
                let field = &self.fields[i];
                write!(f, "{}: {}, ", field.name, field.value)?;
            }
            let field = &self.fields[self.fields.len() - 1];
            write!(f, "{}: {}", field.name, field.value)?;
        } else if !self.fields.is_empty() {
            let field = &self.fields[0];
            write!(f, "{}: {}", field.name, field.value)?;
        }
        write!(f, ")")
    }
}

impl ItemOp for ConstructorItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let mut result = Item::new();
        for field in &self.fields {
            let value = field.value.execute(context)?;
            if let Type::Primitive(value) = value {
                result.set_field(&field.name, value);
            } else {
                return Err(RuntimeMsg(format!(
                    "Cannot assign non-primitive {} to Item field `{}`",
                    value, &field.name
                )));
            }
        }
        Ok(Type::Item(result))
    }
}

pub struct ConstructorItemOpFactory;

impl ItemOpFactory<ConstructorItemOp> for ConstructorItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        tokens.len() > 2 && check_name("Item", &tokens[0]) && tokens[1].is_open_bracket()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<ConstructorItemOp, SyntaxError> {
        assert_name("Item", tokens)?;
        assert_token_raw(Token::OpenBracket, tokens)?;
        let mut field_descriptors = Vec::new();
        while !tokens.is_empty() {
            if tokens[0].is_close_bracket() {
                break;
            }
            let field_name = assert_token(
                |t| match t {
                    Token::Name(n) => Some(n),
                    _ => None,
                },
                Token::Name("field_name".into()),
                tokens,
            )?;
            assert_token_raw(Token::Equals, tokens)?;
            let field_val;
            if find_next_comma(tokens).is_some() {
                field_val = factory.try_build_item_statement(tokens, dict)?;
                assert_token_raw(Token::Comma, tokens)?;
            } else {
                field_val = factory.try_build_item_statement(tokens, dict)?;
            }
            field_descriptors.push(FieldAssignment {
                name: field_name,
                value: field_val,
            });
        }
        assert_token_raw(Token::CloseBracket, tokens)?;
        Ok(ConstructorItemOp {
            fields: field_descriptors,
        })
    }
}

fn find_next_comma(tokens: &VecDeque<Token>) -> Option<usize> {
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
