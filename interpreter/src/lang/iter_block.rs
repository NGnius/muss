#![allow(clippy::new_without_default)]
use core::ops::Deref;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::lang::utility::assert_token_raw;
use crate::lang::LanguageDictionary;
use crate::lang::{BoxedTransformOpFactory, IteratorItem, Op, PseudoOp};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;
//use crate::Item;

const ITEM_VARIABLE_NAME: &str = "item";

pub trait ItemOp: Debug + Display + Send + Sync {
    fn execute(&self, context: &mut Context) -> Result<Type, RuntimeMsg>;
}

pub trait ItemOpFactory<T: Deref<Target = dyn ItemOp> + 'static>: Send + Sync {
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool;

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<T, SyntaxError>;
}

pub struct ItemOpBoxer<
    X: ItemOpFactory<Y> + 'static,
    Y: Deref<Target = dyn ItemOp> + ItemOp + 'static,
> {
    idc: PhantomData<Y>,
    factory: X,
}

impl<X: ItemOpFactory<Y> + 'static, Y: Deref<Target = dyn ItemOp> + ItemOp + 'static>
    ItemOpFactory<Box<dyn ItemOp>> for ItemOpBoxer<X, Y>
{
    fn is_item_op(&self, tokens: &VecDeque<Token>) -> bool {
        self.factory.is_item_op(tokens)
    }

    fn build_item_op(
        &self,
        tokens: &mut VecDeque<Token>,
        factory: &ItemBlockFactory,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn ItemOp>, SyntaxError> {
        Ok(Box::new(self.factory.build_item_op(tokens, factory, dict)?))
    }
}

#[derive(Debug)]
pub struct ItemBlockStatement {
    statements: Vec<Arc<Box<dyn ItemOp>>>,
    iterable: PseudoOp,
    // state
    last_item: Option<PseudoOp>,
}

impl Display for ItemBlockStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}.{{", self.iterable)?;
        if !self.statements.is_empty() {
            writeln!(f)?;
        }
        for statement in self.statements.iter() {
            writeln!(f, "{},", statement)?;
        }
        write!(f, "}}")
    }
}

impl std::clone::Clone for ItemBlockStatement {
    fn clone(&self) -> Self {
        Self {
            statements: Vec::with_capacity(0),
            iterable: self.iterable.clone(),
            last_item: None,
        }
    }
}

impl Op for ItemBlockStatement {
    fn enter(&mut self, ctx: Context) {
        self.iterable.try_real().unwrap().enter(ctx)
    }

    fn escape(&mut self) -> Context {
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

    fn dup(&self) -> Box<dyn Op> {
        /*let mut statements_clone = Vec::with_capacity(self.statements.len());
        for stmt in &self.statements {
            statements_clone.push(stmt.dup());
        }*/
        Box::new(Self {
            statements: self.statements.clone(),
            iterable: PseudoOp::from(self.iterable.try_real_ref().unwrap().dup()),
            // state
            last_item: None,
        })
    }
}

impl Iterator for ItemBlockStatement {
    type Item = IteratorItem;

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
            let old_var = replace_item_var(&mut ctx, Type::Item(item));
            for op in self.statements.iter_mut() {
                match op.execute(&mut ctx) {
                    Ok(_) => {}
                    Err(e) => {
                        #[allow(unused_must_use)]
                        {
                            restore_item_var(&mut ctx, old_var);
                        }
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
                Some(Type::Item(item)) => return Some(Ok(item)),
                Some(Type::Op(mut op)) => {
                    op.enter(real_op.escape());
                    if let Some(item) = op.next() {
                        real_op.enter(op.escape());
                        self.last_item = Some(op.into());
                        return Some(item);
                    } else {
                        real_op.enter(op.escape());
                    }
                }
                Some(x) => {
                    return Some(Err(RuntimeError {
                        line: 0,
                        op: PseudoOp::from_printable(self),
                        msg: format!("Expected `item` like Type::Item(Item[...]), got {}", x),
                    }))
                }
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

pub struct ItemBlockFactory {
    vocabulary: Vec<Box<dyn ItemOpFactory<Box<dyn ItemOp>>>>,
}

impl ItemBlockFactory {
    pub fn push<T: ItemOpFactory<Y> + 'static, Y: Deref<Target = dyn ItemOp> + ItemOp + 'static>(
        mut self,
        factory: T,
    ) -> Self {
        self.vocabulary.push(Box::new(ItemOpBoxer {
            factory: factory,
            idc: PhantomData,
        }));
        self
    }

    pub fn try_build_item_statement(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn ItemOp>, SyntaxError> {
        //for (i, factory) in self.vocabulary.iter().enumerate() {
        for factory in self.vocabulary.iter() {
            if factory.is_item_op(tokens) {
                return factory.build_item_op(tokens, self, dict);
            }
        }
        Err(match tokens.pop_front() {
            Some(x) => SyntaxError {
                line: 0,
                token: Token::Name("{any item op}".into()),
                got: Some(x),
            },
            None => SyntaxError {
                line: 0,
                token: Token::Name("{item op}".into()),
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

impl BoxedTransformOpFactory for ItemBlockFactory {
    fn build_transform_op(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
        op: Box<dyn Op>,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        assert_token_raw(Token::Dot, tokens)?;
        assert_token_raw(Token::OpenCurly, tokens)?;
        let mut item_ops = Vec::with_capacity(tokens.len() / 8);
        while !tokens.is_empty() {
            if tokens[0].is_close_curly() {
                break;
            }
            item_ops.push(Arc::new(self.try_build_item_statement(tokens, dict)?));
            if !tokens.is_empty() {
                if tokens[0].is_comma() {
                    assert_token_raw(Token::Comma, tokens)?;
                }
            } else {
                return Err(SyntaxError {
                    got: tokens.pop_front(),
                    token: Token::Literal(", or }".into()),
                    line: 0,
                })
            }
        }
        assert_token_raw(Token::CloseCurly, tokens)?;
        Ok(Box::new(ItemBlockStatement {
            statements: item_ops,
            iterable: op.into(),
            last_item: None,
        }))
    }

    fn is_transform_op(&self, tokens: &VecDeque<Token>) -> bool {
        tokens.len() > 1 && tokens[0].is_dot() && tokens[1].is_open_curly()
    }
}

fn replace_item_var(ctx: &mut Context, item: Type) -> Option<Type> {
    // remove existing item variable if exists
    let old_var = if ctx.variables.exists(ITEM_VARIABLE_NAME) {
        ctx.variables.remove(ITEM_VARIABLE_NAME).ok()
    } else {
        None
    };
    ctx.variables.declare(ITEM_VARIABLE_NAME, item).unwrap();
    old_var
}

fn restore_item_var(ctx: &mut Context, old_var: Option<Type>) -> Result<Option<Type>, RuntimeMsg> {
    let new_var = if ctx.variables.exists(ITEM_VARIABLE_NAME) {
        Some(ctx.variables.remove(ITEM_VARIABLE_NAME)?)
    } else {
        None
    };
    if let Some(old_var) = old_var {
        ctx.variables.declare(ITEM_VARIABLE_NAME, old_var)?;
    }
    Ok(new_var)
}

/*fn find_next_comma(tokens: &VecDeque<Token>) -> Option<usize> {
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
}*/
