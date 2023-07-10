use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::SingleItem;
use crate::lang::FilterPredicate;
use crate::lang::{IteratorItem, Op, PseudoOp};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp};
use crate::processing::general::Type;
use crate::Context;
use crate::Item;

const ITEM_VARIABLE_NAME: &str = "item";
const ITEM_CACHE_DEFAULT_SIZE: usize = 8;

#[inline(always)]
pub(super) fn item_cache_deque() -> VecDeque<Result<Item, RuntimeError>> {
    VecDeque::with_capacity(ITEM_CACHE_DEFAULT_SIZE)
}

#[derive(Debug)]
pub struct FilterReplaceStatement<P: FilterPredicate + 'static> {
    pub(super) predicate: P,
    pub(super) iterable: PseudoOp,
    pub(super) context: Option<Context>,
    pub(super) op_if: PseudoOp,
    pub(super) op_else: Option<PseudoOp>,
    pub(super) item_cache: VecDeque<Result<Item, RuntimeError>>,
}

impl<P: FilterPredicate + 'static> std::clone::Clone for FilterReplaceStatement<P> {
    fn clone(&self) -> Self {
        Self {
            predicate: self.predicate.clone(),
            iterable: self.iterable.clone(),
            context: None,
            op_if: self.op_if.clone(),
            op_else: self.op_else.clone(),
            item_cache: VecDeque::new(), // this doesn't need to be carried around
        }
    }
}

impl<P: FilterPredicate + 'static> Display for FilterReplaceStatement<P> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(op_else) = &self.op_else {
            write!(
                f,
                "{}.(if {}: {} else {})",
                self.iterable, self.predicate, self.op_if, op_else
            )
        } else {
            write!(
                f,
                "{}.(if {}: {})",
                self.iterable, self.predicate, self.op_if
            )
        }
    }
}

impl<P: FilterPredicate + 'static> Op for FilterReplaceStatement<P> {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        match &self.iterable {
            PseudoOp::Real(op) => op.is_resetable(),
            PseudoOp::Fake(_) => false,
        }
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.item_cache.clear();
        let fake = PseudoOp::Fake(format!("{}", self));
        self.predicate
            .reset()
            .map_err(|x| x.with(RuntimeOp(fake.clone())))?;
        match &mut self.iterable {
            PseudoOp::Real(op) => {
                op.enter(self.context.take().unwrap());
                let result = op.reset();
                self.context = Some(op.escape());
                result
            }
            PseudoOp::Fake(_) => Err(RuntimeError {
                line: 0,
                op: fake,
                msg: "Cannot reset PseudoOp::Fake filter".to_string(),
            }),
        }
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            predicate: self.predicate.clone(),
            iterable: self.iterable.try_real_ref().unwrap().dup().into(),
            context: None,
            op_if: PseudoOp::from(self.op_if.try_real_ref().unwrap().dup()),
            op_else: self
                .op_else
                .as_ref()
                .map(|x| PseudoOp::from(x.try_real_ref().unwrap().dup())),
            item_cache: VecDeque::new(),
        })
    }
}

impl<P: FilterPredicate + 'static> Iterator for FilterReplaceStatement<P> {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.item_cache.is_empty() {
            return self.item_cache.pop_front();
        }
        let fake = PseudoOp::Fake(format!("{}", self));
        // get next item in iterator
        let next_item = match self.iterable.try_real() {
            Ok(real_op) => {
                let ctx = self.context.take().unwrap();
                real_op.enter(ctx);
                let item = real_op.next();
                self.context = Some(real_op.escape());
                item
            }
            Err(e) => return Some(Err(e)),
        };
        // process item
        match next_item {
            Some(Ok(item)) => {
                //println!("item is now: `{}`", &item.filename);
                match self
                    .predicate
                    .matches(&item, self.context.as_mut().unwrap())
                {
                    Ok(is_match) => {
                        if is_match {
                            // unwrap inner operation
                            match self.op_if.try_real() {
                                Ok(real_op) => {
                                    // build item variable
                                    let single_op = SingleItem::new_ok(item);
                                    //println!("Declaring item variable");
                                    let old_item = match declare_or_replace_item(
                                        single_op,
                                        self.context.as_mut().unwrap(),
                                    ) {
                                        Ok(x) => x,
                                        Err(e) => return Some(Err(e.with(RuntimeOp(fake)))), // probably shouldn't occur
                                    };
                                    // invoke inner op
                                    real_op.enter(self.context.take().unwrap());
                                    if real_op.is_resetable() {
                                        if let Err(e) = real_op.reset() {
                                            self.context = Some(real_op.escape());
                                            return Some(Err(e));
                                        }
                                    }
                                    for item in real_op.by_ref() {
                                        self.item_cache.push_back(item);
                                    }
                                    self.context = Some(real_op.escape());
                                    // destroy item variable
                                    //println!("Removing item variable");
                                    match remove_or_replace_item(
                                        old_item,
                                        self.context.as_mut().unwrap(),
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => return Some(Err(e.with(RuntimeOp(fake)))),
                                    }
                                }
                                Err(e) => return Some(Err(e)), // probably shouldn't occur
                            }
                            // return cached item, if any
                            let replacement = self.item_cache.pop_front();
                            if replacement.is_none() {
                                self.next()
                            } else {
                                replacement
                            }
                        } else if let Some(op_else) = &mut self.op_else {
                            println!("op_else {}\n{:?}", op_else, op_else);
                            // unwrap inner operation
                            match op_else.try_real() {
                                Ok(real_op) => {
                                    // build item variable
                                    let single_op = SingleItem::new_ok(item);
                                    //println!("Declaring item variable");
                                    let old_item = match declare_or_replace_item(
                                        single_op,
                                        self.context.as_mut().unwrap(),
                                    ) {
                                        Ok(x) => x,
                                        Err(e) => return Some(Err(e.with(RuntimeOp(fake)))), // probably shouldn't occur
                                    };
                                    // invoke inner operation
                                    real_op.enter(self.context.take().unwrap());
                                    if real_op.is_resetable() {
                                        if let Err(e) = real_op.reset() {
                                            self.context = Some(real_op.escape());
                                            return Some(Err(e));
                                        }
                                    }
                                    for item in real_op.by_ref() {
                                        self.item_cache.push_back(item);
                                    }
                                    self.context = Some(real_op.escape());
                                    // destroy item variable
                                    //println!("Removing item variable");
                                    match remove_or_replace_item(
                                        old_item,
                                        self.context.as_mut().unwrap(),
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => return Some(Err(e.with(RuntimeOp(fake)))),
                                    }
                                }
                                Err(e) => return Some(Err(e)), // probably shouldn't occur
                            }
                            // return cached item, if any
                            let replacement = self.item_cache.pop_front();
                            if replacement.is_none() {
                                self.next()
                            } else {
                                replacement
                            }
                        } else {
                            Some(Ok(item))
                        }
                    }
                    Err(e) => Some(Err(e.with(RuntimeOp(fake)))),
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterable.try_real_ref().map(|x| x.size_hint()).ok().unwrap_or((0, None))
    }
}

fn declare_or_replace_item(
    single: SingleItem,
    ctx: &mut Context,
) -> Result<Option<Type>, RuntimeMsg> {
    let old_item: Option<Type> = if ctx.variables.exists(ITEM_VARIABLE_NAME) {
        Some(ctx.variables.remove(ITEM_VARIABLE_NAME)?)
    } else {
        None
    };
    ctx.variables
        .declare(ITEM_VARIABLE_NAME, Type::Op(Box::new(single)))?;
    Ok(old_item)
}

fn remove_or_replace_item(old_item: Option<Type>, ctx: &mut Context) -> Result<(), RuntimeMsg> {
    ctx.variables.remove(ITEM_VARIABLE_NAME)?;
    if let Some(old_item) = old_item {
        ctx.variables.declare(ITEM_VARIABLE_NAME, old_item)?;
    }
    Ok(())
}
