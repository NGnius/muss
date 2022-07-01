use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::SingleItem;
use crate::lang::{filter::VariableOrOp, FilterPredicate};
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
    pub(super) iterable: VariableOrOp,
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
            VariableOrOp::Variable(s) => {
                if self.context.is_some() {
                    let var = self.context.as_ref().unwrap().variables.get_opt(s);
                    if let Some(Type::Op(var)) = var {
                        var.is_resetable()
                    } else {
                        false
                    }
                } else {
                    true
                } // ASSUMPTION
            }
            VariableOrOp::Op(PseudoOp::Real(op)) => op.is_resetable(),
            VariableOrOp::Op(PseudoOp::Fake(_)) => false,
        }
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.item_cache.clear();
        let fake = PseudoOp::Fake(format!("{}", self));
        self.predicate
            .reset()
            .map_err(|x| x.with(RuntimeOp(fake.clone())))?;
        match &mut self.iterable {
            VariableOrOp::Variable(s) => {
                if self.context.as_mut().unwrap().variables.exists(s) {
                    let mut var = self
                        .context
                        .as_mut()
                        .unwrap()
                        .variables
                        .remove(s)
                        .map_err(|e| e.with(RuntimeOp(fake.clone())))?;
                    let result = if let Type::Op(var) = &mut var {
                        var.enter(self.context.take().unwrap());
                        let result = var.reset();
                        self.context = Some(var.escape());
                        result
                    } else {
                        Err(RuntimeError {
                            line: 0,
                            op: fake.clone(),
                            msg: "Cannot reset non-iterable filter variable".to_string(),
                        })
                    };
                    self.context
                        .as_mut()
                        .unwrap()
                        .variables
                        .declare(s, var)
                        .map_err(|e| e.with(RuntimeOp(fake)))?;
                    result
                } else {
                    Ok(())
                }
            }
            VariableOrOp::Op(PseudoOp::Real(op)) => {
                op.enter(self.context.take().unwrap());
                let result = op.reset();
                self.context = Some(op.escape());
                result
            }
            VariableOrOp::Op(PseudoOp::Fake(_)) => Err(RuntimeError {
                line: 0,
                op: fake,
                msg: "Cannot reset PseudoOp::Fake filter".to_string(),
            }),
        }
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            predicate: self.predicate.clone(),
            iterable: match &self.iterable {
                VariableOrOp::Variable(s) => VariableOrOp::Variable(s.clone()),
                VariableOrOp::Op(op) => VariableOrOp::Op(op.try_real_ref().unwrap().dup().into()),
            },
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
        let next_item = match &mut self.iterable {
            VariableOrOp::Op(op) => match op.try_real() {
                Ok(real_op) => {
                    let ctx = self.context.take().unwrap();
                    real_op.enter(ctx);
                    let item = real_op.next();
                    self.context = Some(real_op.escape());
                    item
                }
                Err(e) => return Some(Err(e)),
            },
            VariableOrOp::Variable(variable_name) => {
                let mut variable = match self
                    .context
                    .as_mut()
                    .unwrap()
                    .variables
                    .remove(variable_name)
                {
                    Ok(Type::Op(op)) => op,
                    Ok(x) => {
                        return Some(Err(RuntimeError {
                            line: 0,
                            op: fake.clone(),
                            msg: format!(
                                "Expected operation/iterable type in variable {}, got {}",
                                &variable_name, x
                            ),
                        }))
                    }
                    Err(e) => return Some(Err(e.with(RuntimeOp(fake)))),
                };
                let ctx = self.context.take().unwrap();
                variable.enter(ctx);
                let item = variable.next();
                self.context = Some(variable.escape());
                match self
                    .context
                    .as_mut()
                    .unwrap()
                    .variables
                    .declare(variable_name, Type::Op(variable))
                {
                    Err(e) => return Some(Err(e.with(RuntimeOp(fake)))),
                    Ok(_) => {}
                }
                item
            }
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
                                        match real_op.reset() {
                                            Err(e) => return Some(Err(e)),
                                            Ok(_) => {}
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
                                        match real_op.reset() {
                                            Err(e) => return Some(Err(e)),
                                            Ok(_) => {}
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
                    Err(e) => Some(Err(e.with(RuntimeOp(fake.clone())))),
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.iterable {
            VariableOrOp::Variable(s) => self
                .context
                .as_ref()
                .and_then(|x| x.variables.get_opt(s))
                .and_then(|x| match x {
                    Type::Op(op) => Some(op.size_hint()),
                    _ => None,
                }),
            VariableOrOp::Op(op) => op.try_real_ref().map(|x| x.size_hint()).ok(),
        }
        .unwrap_or((0, None))
    }
}

fn declare_or_replace_item(
    single: SingleItem,
    ctx: &mut Context,
) -> Result<Option<Type>, RuntimeMsg> {
    let old_item: Option<Type>;
    if ctx.variables.exists(ITEM_VARIABLE_NAME) {
        old_item = Some(ctx.variables.remove(ITEM_VARIABLE_NAME)?);
    } else {
        old_item = None;
    }
    ctx.variables
        .declare(ITEM_VARIABLE_NAME, Type::Op(Box::new(single)))?;
    Ok(old_item)
}

fn remove_or_replace_item(
    old_item: Option<Type>,
    ctx: &mut Context,
) -> Result<(), RuntimeMsg> {
    ctx.variables.remove(ITEM_VARIABLE_NAME)?;
    if let Some(old_item) = old_item {
        ctx.variables.declare(ITEM_VARIABLE_NAME, old_item)?;
    }
    Ok(())
}
