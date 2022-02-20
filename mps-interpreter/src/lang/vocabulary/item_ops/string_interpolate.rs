use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token_raw, assert_token};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{MpsItemOp, MpsItemOpFactory, MpsItemBlockFactory, MpsTypePrimitive};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct InterpolateStringItemOp {
    format: String,
    inner_op: Box<dyn MpsItemOp>,
}

impl Deref for InterpolateStringItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for InterpolateStringItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "~ `{}` {}", &self.format, self.inner_op)
    }
}

impl MpsItemOp for InterpolateStringItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let inner_val = self.inner_op.execute(context)?;
        match inner_val {
            MpsType::Primitive(val) => {
                let result = self.format.replace("{}", &val.as_str());
                Ok(MpsType::Primitive(MpsTypePrimitive::String(result)))
            },
            MpsType::Item(item) => {
                let mut result;
                if item.len() == 0 {
                    result = self.format.clone();
                } else {
                    let mut iter = item.iter();
                    let field1 = iter.next().unwrap();
                    result = self.format.replace(&format!("{{{}}}", field1), &item.field(field1).unwrap().as_str());
                    for field in iter {
                        result = result.replace(&format!("{{{}}}", field), &item.field(field).unwrap().as_str());
                    }
                }
                Ok(MpsType::Primitive(MpsTypePrimitive::String(result)))
            },
            MpsType::Op(op) => {
                let result = self.format.replace("{}", &format!("{}", op));
                Ok(MpsType::Primitive(MpsTypePrimitive::String(result)))
            },
            //val => Err(RuntimeMsg(format!("Cannot insert {} ({}) into format string", self.inner_op, val)))
        }
        //Ok(MpsType::empty())
    }
}

pub struct InterpolateStringItemOpFactory;

impl MpsItemOpFactory<InterpolateStringItemOp> for InterpolateStringItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        !tokens.is_empty()
        && tokens[0].is_tilde()
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<InterpolateStringItemOp, SyntaxError> {
        assert_token_raw(MpsToken::Tilde, tokens)?;
        let format_str = assert_token(|t| match t {
            MpsToken::Literal(s) => Some(s),
            _ => None,
        }, MpsToken::Literal("format_string".into()), tokens)?;
        let inner = factory.try_build_item_statement(tokens, dict)?;
        Ok(InterpolateStringItemOp {
            format: format_str,
            inner_op: inner
        })
    }
}
