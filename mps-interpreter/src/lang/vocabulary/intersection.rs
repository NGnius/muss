use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::MpsToken;
use crate::MpsContext;

use crate::lang::repeated_tokens;
use crate::lang::vocabulary::union::next_comma;
use crate::lang::{MpsFunctionFactory, MpsFunctionStatementFactory, MpsIteratorItem, MpsOp};
use crate::lang::{MpsLanguageDictionary, PseudoOp};
use crate::lang::{RuntimeError, SyntaxError};

#[derive(Debug)]
pub struct IntersectionStatement {
    context: Option<MpsContext>,
    ops: Vec<PseudoOp>,
    items: Option<HashSet<MpsIteratorItem>>,
    original_order: Option<VecDeque<MpsIteratorItem>>,
    init_needed: bool,
}

impl Display for IntersectionStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut ops_str = "".to_owned();
        for i in 0..self.ops.len() {
            ops_str += &self.ops[i].to_string();
            if i != self.ops.len() - 1 {
                ops_str += ", ";
            }
        }
        write!(f, "intersection({})", ops_str)
    }
}

impl std::clone::Clone for IntersectionStatement {
    fn clone(&self) -> Self {
        Self {
            context: None,
            ops: self.ops.clone(),
            items: None,
            original_order: None,
            init_needed: self.init_needed,
        }
    }
}

impl Iterator for IntersectionStatement {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ops.len() == 0 {
            return None;
        } else if self.init_needed {
            self.init_needed = false;
            let real_op = match self.ops[0].try_real() {
                Ok(op) => op,
                Err(e) => return Some(Err(e)),
            };
            real_op.enter(self.context.take().unwrap());
            let original_order: VecDeque<MpsIteratorItem> = real_op.collect();
            let mut set: HashSet<MpsIteratorItem> =
                original_order.iter().map(|x| x.to_owned()).collect();
            self.context = Some(real_op.escape());
            if self.ops.len() != 1 && !set.is_empty() {
                for i in 1..self.ops.len() {
                    let real_op = match self.ops[i].try_real() {
                        Ok(op) => op,
                        Err(e) => return Some(Err(e)),
                    };
                    real_op.enter(self.context.take().unwrap());
                    let set2: HashSet<MpsIteratorItem> = real_op.collect();
                    self.context = Some(real_op.escape());
                    set.retain(|item| set2.contains(item));
                }
            }
            self.original_order = Some(original_order);
            self.items = Some(set);
            self.init_needed = false;
        }
        let original_order = self.original_order.as_mut().unwrap();
        let set_items = self.items.as_ref().unwrap();
        while let Some(item) = original_order.pop_front() {
            if set_items.contains(&item) {
                return Some(item);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl MpsOp for IntersectionStatement {
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
        self.init_needed = true;
        self.original_order = None;
        self.items = None;
        for op in &mut self.ops {
            let real_op = op.try_real()?;
            real_op.enter(self.context.take().unwrap());
            if real_op.is_resetable() {
                let result = real_op.reset();
                self.context = Some(real_op.escape());
                result?;
            } else {
                self.context = Some(real_op.escape());
            }
        }
        Ok(())
    }

    fn dup(&self) -> Box<dyn MpsOp> {
        let mut clone = Self {
            context: None,
            ops: Vec::with_capacity(self.ops.len()),
            items: None,
            original_order: None,
            init_needed: true,
        };
        for op in self.ops.iter() {
            clone.ops.push(PseudoOp::from(op.try_real_ref().unwrap().dup()));
        }
        Box::new(clone)
    }
}

pub struct IntersectionFunctionFactory;

impl MpsFunctionFactory<IntersectionStatement> for IntersectionFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "intersection" || name == "n"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<IntersectionStatement, SyntaxError> {
        // intersection(op1, op2, ...)
        let operations = repeated_tokens(
            |tokens| {
                if let Some(comma_pos) = next_comma(tokens) {
                    let end_tokens = tokens.split_off(comma_pos);
                    let op = dict.try_build_statement(tokens);
                    tokens.extend(end_tokens);
                    Ok(Some(PseudoOp::from(op?)))
                } else {
                    Ok(Some(PseudoOp::from(dict.try_build_statement(tokens)?)))
                }
            },
            MpsToken::Comma,
        )
        .ingest_all(tokens)?;
        Ok(IntersectionStatement {
            context: None,
            ops: operations,
            items: None,
            original_order: None,
            init_needed: true,
        })
    }
}

pub type IntersectionStatementFactory =
    MpsFunctionStatementFactory<IntersectionStatement, IntersectionFunctionFactory>;

#[inline(always)]
pub fn intersection_function_factory() -> IntersectionStatementFactory {
    IntersectionStatementFactory::new(IntersectionFunctionFactory)
}
