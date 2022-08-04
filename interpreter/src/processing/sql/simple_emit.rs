use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::Context;

use crate::lang::{IteratorItem, Op, PseudoOp};
use crate::lang::{RuntimeError, RuntimeOp, TypePrimitive};
use crate::processing::general::FileIter;

#[derive(Debug)]
pub struct SimpleSqlQuery {
    context: Option<Context>,
    file_iter: Option<FileIter>,
    field_name: String,
    val: String,
    has_tried: bool,
}

impl SimpleSqlQuery {
    pub fn emit(field: &str, value: &str) -> Self {
        Self {
            context: None,
            file_iter: None,
            field_name: field.to_owned(),
            val: crate::lang::vocabulary::filters::utility::sanitise_string(value),
            has_tried: false,
        }
    }
}

impl Display for SimpleSqlQuery {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}(`{}`)", self.field_name, self.val)
    }
}

impl std::clone::Clone for SimpleSqlQuery {
    fn clone(&self) -> Self {
        Self {
            context: None,
            file_iter: None,
            field_name: self.field_name.clone(),
            val: self.val.clone(),
            has_tried: self.has_tried,
        }
    }
}

impl Iterator for SimpleSqlQuery {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.file_iter.is_none() {
            if self.has_tried {
                return None;
            } else {
                self.has_tried = true;
            }
            let iter = self.context.as_mut().unwrap().filesystem.raw(
                None,
                None,
                true,
            );
            self.file_iter = Some(match iter {
                Ok(x) => x,
                Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            });
        }
        while let Some(item) = self.file_iter.as_mut().unwrap().next() {
            match item {
                Ok(item) => {
                    // apply filter
                    if let Some(TypePrimitive::String(field_val)) = item.field(&self.field_name) {
                        if crate::lang::vocabulary::filters::utility::sanitise_string(field_val).contains(&self.val) {
                            return Some(Ok(item));
                        }
                    }
                },
                Err(e) => return Some(Err(RuntimeError {
                    line: 0,
                    op: PseudoOp::from_printable(self),
                    msg: e,
                }))
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.file_iter.as_ref().map(|x| x.size_hint()).unwrap_or_default()
    }
}

impl Op for SimpleSqlQuery {
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
