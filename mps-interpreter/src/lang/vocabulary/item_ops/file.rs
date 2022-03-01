use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_name, assert_token_raw, check_name};
use crate::lang::{MpsItemBlockFactory, MpsItemOp, MpsItemOpFactory};
use crate::lang::{MpsLanguageDictionary, MpsTypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;

#[derive(Debug)]
pub struct FileItemOp {
    inner: Box<dyn MpsItemOp>,
}

impl Deref for FileItemOp {
    type Target = dyn MpsItemOp;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl Display for FileItemOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "file({})", self.inner)
    }
}

impl MpsItemOp for FileItemOp {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg> {
        let inner_return = self.inner.execute(context)?;
        if let MpsType::Primitive(MpsTypePrimitive::String(path)) = inner_return {
            Ok(MpsType::Item(context.filesystem.single(&path, None)?))
        } else {
            Err(RuntimeMsg(format!(
                "Cannot use {} as filepath (should be String)",
                inner_return
            )))
        }
    }
}

pub struct FileItemOpFactory;

impl MpsItemOpFactory<FileItemOp> for FileItemOpFactory {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        !tokens.is_empty() && check_name("file", &tokens[0])
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<FileItemOp, SyntaxError> {
        assert_name("file", tokens)?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        let end_tokens = tokens.split_off(tokens.len() - 1);
        let inner_op = factory.try_build_item_statement(tokens, dict)?;
        tokens.extend(end_tokens);
        assert_token_raw(MpsToken::CloseBracket, tokens)?;
        Ok(FileItemOp { inner: inner_op })
    }
}
