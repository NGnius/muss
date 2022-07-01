use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::LanguageDictionary;
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory, TypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct InterpolateStringItemOp {
    format: String,
    inner_op: Box<dyn ItemOp>,
}

impl Deref for InterpolateStringItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for InterpolateStringItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "~ `{}` {}", &self.format, self.inner_op)
    }
}

impl ItemOp for InterpolateStringItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let inner_val = self.inner_op.execute(context)?;
        match inner_val {
            Type::Primitive(val) => {
                let result = self.format.replace("{}", &val.as_str());
                Ok(Type::Primitive(TypePrimitive::String(result)))
            }
            Type::Item(item) => {
                let mut result;
                if item.len() == 0 {
                    result = self.format.clone();
                } else {
                    let mut iter = item.iter();
                    let field1 = iter.next().unwrap();
                    result = self.format.replace(
                        &format!("{{{}}}", field1),
                        &item.field(field1).unwrap().as_str(),
                    );
                    for field in iter {
                        result = result.replace(
                            &format!("{{{}}}", field),
                            &item.field(field).unwrap().as_str(),
                        );
                    }
                }
                Ok(Type::Primitive(TypePrimitive::String(result)))
            }
            Type::Op(op) => {
                let result = self.format.replace("{}", &format!("{}", op));
                Ok(Type::Primitive(TypePrimitive::String(result)))
            } //val => Err(RuntimeMsg(format!("Cannot insert {} ({}) into format string", self.inner_op, val)))
        }
        //Ok(Type::empty())
    }
}

pub struct InterpolateStringItemOpFactory;

impl ItemOpFactory<InterpolateStringItemOp> for InterpolateStringItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        !tokens.is_empty() && tokens[0].is_tilde()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<InterpolateStringItemOp, SyntaxError> {
        assert_token_raw(Token::Tilde, tokens)?;
        let format_str = assert_token(
            |t| match t {
                Token::Literal(s) => Some(s),
                _ => None,
            },
            Token::Literal("format_string".into()),
            tokens,
        )?;
        let inner = factory.try_build_item_statement(tokens, dict)?;
        Ok(InterpolateStringItemOp {
            format: format_str,
            inner_op: inner,
        })
    }
}
