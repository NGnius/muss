use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::Token;
use crate::Context;

use crate::lang::LanguageDictionary;
use crate::lang::{FunctionFactory, FunctionStatementFactory, IteratorItem, Op};
use crate::lang::{RuntimeError, SyntaxError};

#[derive(Debug)]
pub struct EmptyStatement {
    pub(crate) context: Option<Context>,
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
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl Op for EmptyStatement {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(self.clone())
    }
}

pub struct EmptyFunctionFactory;

impl FunctionFactory<EmptyStatement> for EmptyFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "empty" || name == "_"
    }

    fn build_function_params(
        &self,
        _name: String,
        _tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<EmptyStatement, SyntaxError> {
        // empty()
        Ok(EmptyStatement { context: None })
    }
}

pub type EmptyStatementFactory = FunctionStatementFactory<EmptyStatement, EmptyFunctionFactory>;

#[inline(always)]
pub fn empty_function_factory() -> EmptyStatementFactory {
    EmptyStatementFactory::new(EmptyFunctionFactory)
}
