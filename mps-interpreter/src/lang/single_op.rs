use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::{MpsIteratorItem, MpsOp, RuntimeError};
use crate::MpsContext;
use crate::MpsItem;

#[derive(Debug)]
pub struct SingleItem {
    context: Option<MpsContext>,
    item: Result<MpsItem, RuntimeError>,
    is_complete: bool,
}

impl SingleItem {
    pub fn new(item: Result<MpsItem, RuntimeError>) -> Self {
        Self {
            context: None,
            item,
            is_complete: false,
        }
    }

    pub fn new_ok(item: MpsItem) -> Self {
        Self::new(Ok(item))
    }

    pub fn replace(&mut self, new_item: Result<MpsItem, RuntimeError>) {
        self.item = new_item
    }
}

impl Display for SingleItem {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match &self.item {
            Ok(item) => write!(f, "*single item*[Ok({})]", item),
            Err(e) => write!(f, "*single-item*[Err({})]", e),
        }
    }
}

impl std::clone::Clone for SingleItem {
    fn clone(&self) -> Self {
        Self {
            context: None,
            item: self.item.clone(),
            is_complete: self.is_complete,
        }
    }
}

impl Iterator for SingleItem {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_complete {
            None
        } else {
            self.is_complete = true;
            Some(self.item.clone())
        }
    }
}

impl MpsOp for SingleItem {
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
        self.is_complete = false;
        Ok(())
    }

    fn dup(&self) -> Box<dyn MpsOp> {
        Box::new(Self {
            context: None,
            item: self.item.clone(),
            is_complete: false,
        })
    }
}
