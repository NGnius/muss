use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_name, assert_token_raw, check_name};
use crate::lang::{ItemBlockFactory, ItemOp, ItemOpFactory};
use crate::lang::{LanguageDictionary, TypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub struct FileItemOp {
    inner: Box<dyn ItemOp>,
}

impl Deref for FileItemOp {
    type Target = dyn ItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for FileItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "file({})", self.inner)
    }
}

impl ItemOp for FileItemOp {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg> {
        let inner_return = self.inner.execute(context)?;
        if let Type::Primitive(TypePrimitive::String(path)) = inner_return {
            Ok(Type::Item(context.filesystem.single(&path, None)?))
        } else {
            Err(RuntimeMsg(format!(
                "Cannot use {} as filepath (should be String)",
                inner_return
            )))
        }
    }
}

pub struct FileItemOpFactory;

impl ItemOpFactory<FileItemOp> for FileItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        !tokens.is_empty() && check_name("file", &tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<FileItemOp, SyntaxError> {
        assert_name("file", tokens)?;
        assert_token_raw(Token::OpenBracket, tokens)?;
        let end_tokens = tokens.split_off(tokens.len() - 1);
        let inner_op = factory.try_build_item_statement(tokens, dict)?;
        tokens.extend(end_tokens);
        assert_token_raw(Token::CloseBracket, tokens)?;
        Ok(FileItemOp { inner: inner_op })
    }
}
