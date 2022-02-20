use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::MpsToken;
use crate::MpsContext;

use crate::lang::MpsLanguageDictionary;
use crate::lang::PseudoOp;
use crate::lang::{MpsFunctionFactory, MpsFunctionStatementFactory, MpsIteratorItem, MpsOp};
use crate::lang::{RuntimeError, SyntaxError};

#[derive(Debug)]
pub struct ResetStatement {
    context: Option<MpsContext>,
    inner: PseudoOp,
    // state
    has_tried: bool,
}

impl Display for ResetStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "reset({})", &self.inner)
    }
}

impl std::clone::Clone for ResetStatement {
    fn clone(&self) -> Self {
        Self {
            context: None,
            inner: self.inner.clone(),
            has_tried: self.has_tried,
        }
    }
}

impl Iterator for ResetStatement {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_tried {
            self.has_tried = true;
            let inner = match self.inner.try_real() {
                Ok(x) => x,
                Err(e) => return Some(Err(e)),
            };
            match inner.reset() {
                Ok(_) => {}
                Err(e) => return Some(Err(e)),
            };
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl MpsOp for ResetStatement {
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
        self.has_tried = false;
        Ok(())
    }

    fn dup(&self) -> Box<dyn MpsOp> {
        Box::new(Self {
            context: None,
            inner: PseudoOp::from(self.inner.try_real_ref().unwrap().dup()),
            has_tried: false,
        })
    }
}

pub struct ResetFunctionFactory;

impl MpsFunctionFactory<ResetStatement> for ResetFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "reset"
    }

    fn build_function_params(
        &self,
        _name: String,
        #[allow(unused_variables)] tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<ResetStatement, SyntaxError> {
        // reset(var)
        let inner_op = dict.try_build_statement(tokens)?.into();
        Ok(ResetStatement {
            context: None,
            inner: inner_op,
            has_tried: false,
        })
    }
}

pub type ResetStatementFactory = MpsFunctionStatementFactory<ResetStatement, ResetFunctionFactory>;

#[inline(always)]
pub fn reset_function_factory() -> ResetStatementFactory {
    ResetStatementFactory::new(ResetFunctionFactory)
}
