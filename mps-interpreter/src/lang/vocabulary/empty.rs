use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::MpsToken;
use crate::MpsContext;

use crate::lang::MpsLanguageDictionary;
use crate::lang::{MpsFunctionFactory, MpsFunctionStatementFactory, MpsIteratorItem, MpsOp};
use crate::lang::{RuntimeError, SyntaxError};

#[derive(Debug)]
pub struct EmptyStatement {
    context: Option<MpsContext>,
}

impl Display for EmptyStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "empty()")
    }
}

impl std::clone::Clone for EmptyStatement {
    fn clone(&self) -> Self {
        Self { context: None }
    }
}

impl Iterator for EmptyStatement {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl MpsOp for EmptyStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn dup(&self) -> Box<dyn MpsOp> {
        Box::new(self.clone())
    }
}

pub struct EmptyFunctionFactory;

impl MpsFunctionFactory<EmptyStatement> for EmptyFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "empty" || name == "_"
    }

    fn build_function_params(
        &self,
        _name: String,
        _tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<EmptyStatement, SyntaxError> {
        // empty()
        Ok(EmptyStatement { context: None })
    }
}

pub type EmptyStatementFactory = MpsFunctionStatementFactory<EmptyStatement, EmptyFunctionFactory>;

#[inline(always)]
pub fn empty_function_factory() -> EmptyStatementFactory {
    EmptyStatementFactory::new(EmptyFunctionFactory)
}
