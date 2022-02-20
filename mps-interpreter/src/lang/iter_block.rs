use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;

use crate::lang::utility::{assert_token_raw, assert_token_raw_back};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{BoxedMpsOpFactory, MpsIteratorItem, MpsOp, PseudoOp};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
use crate::processing::general::MpsType;
use crate::tokens::MpsToken;
use crate::MpsContext;
//use crate::MpsItem;

const ITEM_VARIABLE_NAME: &str = "item";

pub trait MpsItemOp: Debug + Display {
    fn execute(&self, context: &mut MpsContext) -> Result<MpsType, RuntimeMsg>;
}

pub trait MpsItemOpFactory<T: Deref<Target = dyn MpsItemOp> + 'static> {
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool;

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<T, SyntaxError>;
}

pub struct MpsItemOpBoxer<X: MpsItemOpFactory<Y> + 'static, Y: Deref<Target = dyn MpsItemOp> + MpsItemOp + 'static> {
    idc: PhantomData<Y>,
    factory: X,
}

impl<X: MpsItemOpFactory<Y> + 'static, Y: Deref<Target = dyn MpsItemOp> + MpsItemOp + 'static> MpsItemOpFactory<Box<dyn MpsItemOp>>
    for MpsItemOpBoxer<X, Y>
{
    fn is_item_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        self.factory.is_item_op(tokens)
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        factory: &MpsItemBlockFactory,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsItemOp>, SyntaxError> {
        Ok(Box::new(self.factory.build_item_op(tokens, factory, dict)?))
    }
}

#[derive(Debug)]
pub struct MpsItemBlockStatement {
    statements: Vec<Box<dyn MpsItemOp>>,
    iterable: PseudoOp,
    // state
    last_item: Option<PseudoOp>,
}

impl Display for MpsItemBlockStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}.{{", self.iterable)?;
        if self.statements.len() > 0 {
            write!(f, "\n")?;
        }
        for statement in self.statements.iter() {
            write!(f, "{}\n", statement)?;
        }
        write!(f, "}}")
    }
}

impl std::clone::Clone for MpsItemBlockStatement {
    fn clone(&self) -> Self {
        Self {
            statements: Vec::with_capacity(0),
            iterable: self.iterable.clone(),
            last_item: None,
        }
    }
}


impl MpsOp for MpsItemBlockStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.iterable.try_real().unwrap().enter(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.iterable.try_real().unwrap().escape()
    }

    fn is_resetable(&self) -> bool {
        if let Ok(iter) = self.iterable.try_real_ref() {
            iter.is_resetable()
        } else {
            false
        }
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.iterable.try_real()?.reset()
    }
}

impl Iterator for MpsItemBlockStatement {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let real_op = match self.iterable.try_real() {
            Ok(op) => op,
            Err(e) => return Some(Err(e)),
        };
        if let Some(last_item) = self.last_item.as_mut() {
            let real_last = match last_item.try_real() {
                Ok(op) => op,
                Err(e) => return Some(Err(e)),
            };
            real_last.enter(real_op.escape());
            let next_item = real_last.next();
            real_op.enter(real_last.escape());
            if let Some(item) = next_item {
                return Some(item);
            } else {
                self.last_item = None;
            }
        }
        while let Some(item) = real_op.next() {
            if let Err(e) = item {
                return Some(Err(e));
            }
            let item = item.unwrap();
            let mut ctx = real_op.escape();
            let old_var = replace_item_var(&mut ctx, MpsType::Item(item));
            for op in self.statements.iter_mut() {
                match op.execute(&mut ctx) {
                    Ok(_) => {},
                    Err(e) => {
                        #[allow(unused_must_use)]
                        {restore_item_var(&mut ctx, old_var);}
                        real_op.enter(ctx);
                        return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))));
                    }
                }
            }
            let item = match restore_item_var(&mut ctx, old_var) {
                Ok(item) => item,
                Err(e) => {
                    real_op.enter(ctx);
                    return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))));
                }
            };
            real_op.enter(ctx);
            match item {
                Some(MpsType::Item(item)) => {
                    return Some(Ok(item))
                },
                Some(MpsType::Op(mut op)) => {
                    op.enter(real_op.escape());
                    if let Some(item) = op.next() {
                        real_op.enter(op.escape());
                        self.last_item = Some(op.into());
                        return Some(item);
                    } else {
                        real_op.enter(op.escape());
                    }
                },
                Some(x) => return Some(Err(RuntimeError {
                    line: 0,
                    op: PseudoOp::from_printable(self),
                    msg: format!("Expected `item` like MpsType::Item(MpsItem[...]), got {}", x),
                })),
                None => {}
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterable
            .try_real_ref()
            .map(|x| x.size_hint())
            .unwrap_or((0, None))
    }
}

pub struct MpsItemBlockFactory {
    vocabulary: Vec<Box<dyn MpsItemOpFactory<Box<dyn MpsItemOp>>>>,
}

impl MpsItemBlockFactory {
    pub fn add<T: MpsItemOpFactory<Y> + 'static, Y: Deref<Target=dyn MpsItemOp> + MpsItemOp + 'static>(
        mut self,
        factory: T
    ) -> Self {
        self.vocabulary.push(Box::new(MpsItemOpBoxer {
            factory: factory,
            idc: PhantomData,
        }));
        self
    }

    pub fn try_build_item_statement(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsItemOp>, SyntaxError> {
        for factory in &self.vocabulary {
            if factory.is_item_op(tokens) {
                return factory.build_item_op(tokens, self, dict);
            }
        }
        Err(match tokens.pop_front() {
            Some(x) => SyntaxError {
                line: 0,
                token: MpsToken::Name("{any item op}".into()),
                got: Some(x),
            },
            None => SyntaxError {
                line: 0,
                token: MpsToken::Name("{item op}".into()),
                got: None,
            },
        })
    }

    pub fn new() -> Self {
        Self {
            vocabulary: Vec::new(),
        }
    }
}

impl BoxedMpsOpFactory for MpsItemBlockFactory {
    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens[tokens.len()-1].is_close_curly()
    }

    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsOp>, SyntaxError> {
        let open_curly_pos = if let Some(pos) = find_last_open_curly(tokens) {
            Ok(pos)
        } else {
            Err(SyntaxError {
                line: 0,
                token: MpsToken::OpenCurly,
                got: None,
            })
        }?;
        let block_tokens = tokens.split_off(open_curly_pos - 1); // . always before {
        let inner_op = dict.try_build_statement(tokens)?;
        tokens.extend(block_tokens);
        assert_token_raw(MpsToken::Dot, tokens)?;
        assert_token_raw(MpsToken::OpenCurly, tokens)?;
        assert_token_raw_back(MpsToken::CloseCurly, tokens)?;
        let mut item_ops = Vec::with_capacity(tokens.len() / 8);
        while !tokens.is_empty() {
            if let Some(next_comma) = find_next_comma(tokens) {
                let end_tokens = tokens.split_off(next_comma);
                item_ops.push(self.try_build_item_statement(tokens, dict)?);
                tokens.extend(end_tokens);
                assert_token_raw(MpsToken::Comma, tokens)?;
            } else {
                item_ops.push(self.try_build_item_statement(tokens, dict)?);
            }
        }
        Ok(Box::new(MpsItemBlockStatement {
            statements: item_ops,
            iterable: inner_op.into(),
            last_item: None,
        }))
    }
}

fn replace_item_var(ctx: &mut MpsContext, item: MpsType) -> Option<MpsType> {
    // remove existing item variable if exists
    let old_var = if ctx.variables.exists(ITEM_VARIABLE_NAME) {
        ctx.variables.remove(ITEM_VARIABLE_NAME).ok()
    } else {
        None
    };
    ctx.variables.declare(ITEM_VARIABLE_NAME, item).unwrap();
    old_var
}

fn restore_item_var(ctx: &mut MpsContext, old_var: Option<MpsType>) -> Result<Option<MpsType>, RuntimeMsg> {
    let new_var;
    if ctx.variables.exists(ITEM_VARIABLE_NAME) {
        new_var = Some(ctx.variables.remove(ITEM_VARIABLE_NAME)?);
    } else {
        new_var = None;
    }
    if let Some(old_var) = old_var {
        ctx.variables.declare(ITEM_VARIABLE_NAME, old_var)?;
    }
    Ok(new_var)
}

fn find_last_open_curly(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    let mut curly_found = false;
    for i in (0..tokens.len()).rev() {
        let token = &tokens[i];
        match token {
            MpsToken::OpenCurly => {
                if bracket_depth != 0 {
                    bracket_depth -= 1;
                }
            },
            MpsToken::CloseCurly => {
                bracket_depth += 1;
            },
            MpsToken::Dot => {
                if bracket_depth == 0 && curly_found {
                    return Some(i+1);
                }
            }
            _ => {},
        }
        if token.is_open_curly() {
            curly_found = true;
        } else {
            curly_found = false;
        }
    }
    None
}

fn find_next_comma(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    let mut bracket_depth = 0;
    let mut curly_depth = 0;
    for i in 0..tokens.len() {
        let token = &tokens[i];
        if token.is_comma() && bracket_depth == 0 && curly_depth == 0 {
            return Some(i);
        } else if token.is_open_bracket() {
            bracket_depth += 1;
        } else if token.is_close_bracket() && bracket_depth != 0 {
            bracket_depth -= 1;
        } else if token.is_open_curly() {
            curly_depth += 1;
        } else if token.is_close_curly() && curly_depth != 0 {
            curly_depth -= 1;
        }
    }
    None
}
