use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::{MpsOp, PseudoOp, MpsIteratorItem};
use crate::lang::RuntimeError;
use crate::lang::{MpsFilterPredicate, filter::VariableOrOp};
use crate::lang::SingleItem;
use crate::processing::general::MpsType;
use crate::processing::OpGetter;
use crate::MpsContext;
use crate::MpsItem;

const ITEM_VARIABLE_NAME: &str = "item";
const ITEM_CACHE_DEFAULT_SIZE: usize = 8;

#[inline(always)]
pub(super) fn item_cache_deque() -> VecDeque<Result<MpsItem, RuntimeError>> {
    VecDeque::with_capacity(ITEM_CACHE_DEFAULT_SIZE)
}

#[derive(Debug)]
pub struct MpsFilterReplaceStatement<P: MpsFilterPredicate + 'static> {
    pub(super) predicate: P,
    pub(super) iterable: VariableOrOp,
    pub(super) context: Option<MpsContext>,
    pub(super) op_if: PseudoOp,
    pub(super) op_else: Option<PseudoOp>,
    pub(super) item_cache: VecDeque<Result<MpsItem, RuntimeError>>,
}

impl<P: MpsFilterPredicate + 'static> std::clone::Clone for MpsFilterReplaceStatement<P> {
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

impl<P: MpsFilterPredicate + 'static> Display for MpsFilterReplaceStatement<P> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(op_else) = &self.op_else {
            write!(f, "{}.(if {}: {} else {})", self.iterable, self.predicate, self.op_if, op_else)
        } else {
            write!(f, "{}.(if {}: {})", self.iterable, self.predicate, self.op_if)
        }
    }
}

impl<P: MpsFilterPredicate + 'static> MpsOp for MpsFilterReplaceStatement<P> {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        match &self.iterable {
            VariableOrOp::Variable(s) => {
                if self.context.is_some() {
                    let var = self.context.as_ref().unwrap().variables.get_opt(s);
                    if let Some(MpsType::Op(var)) = var {
                        var.is_resetable()
                    } else {
                        false
                    }
                } else {true} // ASSUMPTION

            }
            VariableOrOp::Op(PseudoOp::Real(op)) => op.is_resetable(),
            VariableOrOp::Op(PseudoOp::Fake(_)) => false,
        }
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.item_cache.clear();
        let fake = PseudoOp::Fake(format!("{}", self));
        self.predicate.reset()?;
        match &mut self.iterable {
            VariableOrOp::Variable(s) => {
                if self.context.as_mut().unwrap().variables.exists(s) {
                    let fake_getter = &mut move || fake.clone();
                    let mut var = self.context.as_mut().unwrap().variables.remove(s, fake_getter)?;
                    let result = if let MpsType::Op(var) = &mut var {
                        var.enter(self.context.take().unwrap());
                        let result = var.reset();
                        self.context = Some(var.escape());
                        result
                    } else {
                        Err(RuntimeError {
                            line: 0,
                            op: fake_getter(),
                            msg: "Cannot reset non-iterable filter variable".to_string(),
                        })
                    };
                    self.context.as_mut().unwrap().variables.declare(s, var, fake_getter)?;
                    result
                } else {Ok(())}
            },
            VariableOrOp::Op(PseudoOp::Real(op)) => {
                op.enter(self.context.take().unwrap());
                let result = op.reset();
                self.context = Some(op.escape());
                result
            },
            VariableOrOp::Op(PseudoOp::Fake(_)) => Err(RuntimeError {
                line: 0,
                op: fake,
                msg: "Cannot reset PseudoOp::Fake filter".to_string(),
            }),
        }
    }
}

impl<P: MpsFilterPredicate + 'static> Iterator for MpsFilterReplaceStatement<P> {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.item_cache.is_empty() {
            return self.item_cache.pop_front();
        }
        let self_clone = self.clone();
        let mut op_getter = move || (Box::new(self_clone.clone()) as Box<dyn MpsOp>).into();
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
                    .remove(&variable_name, &mut op_getter)
                {
                    Ok(MpsType::Op(op)) => op,
                    Ok(x) => {
                        return Some(Err(RuntimeError {
                            line: 0,
                            op: op_getter(),
                            msg: format!(
                                "Expected operation/iterable type in variable {}, got {}",
                                &variable_name, x
                            ),
                        }))
                    }
                    Err(e) => return Some(Err(e)),
                };
                let ctx = self.context.take().unwrap();
                variable.enter(ctx);
                let item = variable.next();
                self.context = Some(variable.escape());
                match self.context.as_mut().unwrap().variables.declare(
                    &variable_name,
                    MpsType::Op(variable),
                    &mut op_getter,
                ) {
                    Err(e) => return Some(Err(e)),
                    Ok(_) => {},
                }
                item
            }
        };
        // process item
        match next_item {
            Some(Ok(item)) => {
                //println!("item is now: `{}`", &item.filename);
                match self.predicate.matches(&item, self.context.as_mut().unwrap(), &mut op_getter) {
                    Ok(is_match) =>
                        if is_match {
                            // unwrap inner operation
                            match self.op_if.try_real() {
                                Ok(real_op) => {
                                    // build item variable
                                    let single_op = SingleItem::new_ok(item);
                                    //println!("Declaring item variable");
                                    let old_item = match declare_or_replace_item(single_op, &mut op_getter, self.context.as_mut().unwrap()) {
                                        Ok(x) => x,
                                        Err(e) => return Some(Err(e)), // probably shouldn't occur
                                    };
                                    // invoke inner op
                                    real_op.enter(self.context.take().unwrap());
                                    if real_op.is_resetable() {
                                        match real_op.reset() {
                                            Err(e) => return Some(Err(e)),
                                            Ok(_) => {}
                                        }
                                    }
                                    while let Some(item) = real_op.next() {
                                        self.item_cache.push_back(item);
                                    }
                                    self.context = Some(real_op.escape());
                                    // destroy item variable
                                    //println!("Removing item variable");
                                    match remove_or_replace_item(old_item, &mut op_getter, self.context.as_mut().unwrap()) {
                                        Ok(_) => {},
                                        Err(e) => return Some(Err(e))
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
                                    let old_item = match declare_or_replace_item(single_op, &mut op_getter, self.context.as_mut().unwrap()) {
                                        Ok(x) => x,
                                        Err(e) => return Some(Err(e)), // probably shouldn't occur
                                    };
                                    // invoke inner operation
                                    real_op.enter(self.context.take().unwrap());
                                    if real_op.is_resetable() {
                                        match real_op.reset() {
                                            Err(e) => return Some(Err(e)),
                                            Ok(_) => {}
                                        }
                                    }
                                    while let Some(item) = real_op.next() {
                                        self.item_cache.push_back(item);
                                    }
                                    self.context = Some(real_op.escape());
                                            // destroy item variable
                                    //println!("Removing item variable");
                                    match remove_or_replace_item(old_item, &mut op_getter, self.context.as_mut().unwrap()) {
                                        Ok(_) => {},
                                        Err(e) => return Some(Err(e))
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
                        },
                    Err(e) => return Some(Err(e))
                }
            },
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}

fn declare_or_replace_item(single: SingleItem, op: &mut OpGetter, ctx: &mut MpsContext) -> Result<Option<MpsType>, RuntimeError> {
    let old_item: Option<MpsType>;
    if ctx.variables.exists(ITEM_VARIABLE_NAME) {
        old_item = Some(ctx.variables.remove(ITEM_VARIABLE_NAME, op)?);
    } else {
        old_item = None;
    }
    ctx.variables.declare(ITEM_VARIABLE_NAME, MpsType::Op(Box::new(single)), op)?;
    Ok(old_item)
}

fn remove_or_replace_item(old_item: Option<MpsType>, op: &mut OpGetter, ctx: &mut MpsContext) -> Result<(), RuntimeError> {
    ctx.variables.remove(ITEM_VARIABLE_NAME, op)?;
    if let Some(old_item) = old_item {
        ctx.variables.declare(ITEM_VARIABLE_NAME, old_item, op)?;
    }
    Ok(())
}
