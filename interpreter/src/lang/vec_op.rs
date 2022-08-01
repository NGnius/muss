use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::{IteratorItem, Op, RuntimeError};
use crate::lang::{RuntimeOp, RuntimeMsg, PseudoOp};
use crate::Context;
use crate::Item;

type IteratorItemMsg = Result<Item, RuntimeMsg>;

#[derive(Debug)]
pub struct VecOp<T> {
    context: Option<Context>,
    vec: Vec<T>,
    index: usize,
}

impl<T> VecOp<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            context: None,
            vec: items,
            index: 0,
        }
    }
}

impl<T> Display for VecOp<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "*vec*[{}..{}]", self.index, self.vec.len())
    }
}

impl<T: Clone> std::clone::Clone for VecOp<T> {
    fn clone(&self) -> Self {
        Self {
            context: None,
            vec: self.vec.clone(),
            index: self.index,
        }
    }
}

impl Iterator for VecOp<IteratorItem> {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.vec.len() {
            None
        } else {
            let item = self.vec[self.index].clone();
            self.index += 1;
            Some(item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.vec.len();
        (len, Some(len))
    }
}

impl Iterator for VecOp<IteratorItemMsg> {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.vec.len() {
            None
        } else {
            let item = self.vec[self.index].clone();
            self.index += 1;
            Some(item.map_err(|e| e.with(RuntimeOp(PseudoOp::from_printable(self)))))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.vec.len();
        (len, Some(len))
    }
}

impl Op for VecOp<IteratorItem> {
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
        self.index = 0;
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            context: None,
            vec: self.vec.clone(),
            index: 0,
        })
    }
}

impl Op for VecOp<IteratorItemMsg> {
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
        self.index = 0;
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            context: None,
            vec: self.vec.clone(),
            index: 0,
        })
    }
}

impl std::convert::From<Vec<IteratorItem>> for VecOp<IteratorItem> {
    fn from(other: Vec<IteratorItem>) -> Self {
        VecOp::new(other)
    }
}

impl std::convert::From<Vec<IteratorItemMsg>> for VecOp<IteratorItemMsg> {
    fn from(other: Vec<IteratorItemMsg>) -> Self {
        VecOp::new(other)
    }
}
