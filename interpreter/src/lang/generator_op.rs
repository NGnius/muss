use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::{IteratorItem, Op, RuntimeError};
use crate::lang::{RuntimeOp, RuntimeMsg, PseudoOp};
use crate::Context;
use crate::Item;

pub struct GeneratorOp {
    context: Option<Context>,
    generator: Box<dyn (FnMut(&mut Context) -> Option<Result<Item, RuntimeMsg>>) + Send>,
}

impl GeneratorOp {
    pub fn new<F: (FnMut(&mut Context) -> Option<Result<Item, RuntimeMsg>>) + Send + 'static>(generator_fn: F) -> Self {
        Self {
            context: None,
            generator: Box::new(generator_fn),
        }
    }
}

impl Display for GeneratorOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "*generator*[...]")
    }
}

impl Debug for GeneratorOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("GeneratorOp")
            .field("context", &self.context)
            .field("generator", &"<boxed function>")
            .finish()
    }
}

impl Iterator for GeneratorOp {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let ctx = self.context.as_mut().unwrap();
        match (self.generator)(ctx) {
            Some(Err(e)) => Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            Some(Ok(item)) => Some(Ok(item)),
            None => None,
        }
    }
}

impl Op for GeneratorOp {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        false
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        Err(
            RuntimeMsg("Cannot reset generator op".to_string())
                .with(RuntimeOp(PseudoOp::from_printable(self)))
        )
    }

    fn dup(&self) -> Box<dyn Op> {
        // this shouldn't be called
        Box::new(Self {
            context: None,
            generator: Box::new(|_| None)
        })
    }
}
