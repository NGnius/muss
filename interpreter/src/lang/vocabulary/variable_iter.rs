use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::Token;
use crate::Context;

use crate::lang::utility::assert_token;
use crate::lang::LanguageDictionary;
use crate::lang::{BoxedOpFactory, IteratorItem, Op, OpFactory, PseudoOp};
use crate::lang::{RuntimeError, RuntimeOp, SyntaxError};
use crate::processing::general::Type;

#[derive(Debug)]
pub struct VariableRetrieveStatement {
    variable_name: String,
    context: Option<Context>,
    is_tried: bool,
}

impl Display for VariableRetrieveStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.variable_name)
    }
}

impl std::clone::Clone for VariableRetrieveStatement {
    fn clone(&self) -> Self {
        Self {
            variable_name: self.variable_name.clone(),
            context: None,
            is_tried: self.is_tried,
        }
    }
}

impl Iterator for VariableRetrieveStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_tried {
            return None;
        }
        let var = self.context.as_mut()
            .unwrap()
            .variables.remove(&self.variable_name)
            .map_err(|e| e.with(RuntimeOp(PseudoOp::from_printable(self))));
        match var {
            Ok(Type::Op(mut op)) => {
                op.enter(self.context.take().unwrap());
                let next_item = op.next();
                self.enter(op.escape());
                if let Err(e) = self.context.as_mut().unwrap().variables.declare(&self.variable_name, Type::Op(op)) {
                    return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))));
                }
                next_item
            },
            Ok(Type::Item(item)) => {
                self.is_tried = true;
                if let Err(e) = self.context.as_mut().unwrap().variables.declare(&self.variable_name, Type::Item(item.clone())) {
                    return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))));
                }
                Some(Ok(item))
            },
            Ok(Type::Primitive(p)) => {
                self.is_tried = true;
                let err_msg = format!("Cannot iterate over primitive `{}` ({})", self.variable_name, p);
                if let Err(e) = self.context.as_mut().unwrap().variables.declare(&self.variable_name, Type::Primitive(p)) {
                    return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))));
                }
                Some(Err(
                    RuntimeError {
                        line: 0,
                        op: PseudoOp::from_printable(self),
                        msg: err_msg,
                    }
                ))
            }
            Err(e) => {
                self.is_tried = true;
                Some(Err(e))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl Op for VariableRetrieveStatement {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        if let Some(ctx) = &self.context {
            let var = ctx.variables.get(&self.variable_name);
            match var {
                Ok(Type::Op(op)) => op.is_resetable(),
                Ok(Type::Primitive(_)) => false,
                Ok(Type::Item(_)) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.is_tried = false;
        let runtime_op = RuntimeOp(PseudoOp::from_printable(self));
        let var = self.context.as_mut()
            .unwrap()
            .variables.get_mut(&self.variable_name)
            .map_err(|e| e.with(runtime_op))?;

        if let Type::Op(op) = var {
            op.reset()?;
        }
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            variable_name: self.variable_name.clone(),
            context: None,
            is_tried: false,
        })
    }
}

pub struct VariableRetrieveStatementFactory;

impl OpFactory<VariableRetrieveStatement> for VariableRetrieveStatementFactory {
    fn is_op(&self, tokens: &VecDeque<Token>) -> bool {
        !tokens.is_empty() && tokens[0].is_name()
    }

    fn build_op(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<VariableRetrieveStatement, SyntaxError> {
        let name = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("variable_name".into()),
            tokens,
        )?;
        Ok(VariableRetrieveStatement {
            variable_name: name,
            context: None,
            is_tried: false,
        })
    }
}

impl BoxedOpFactory for VariableRetrieveStatementFactory {
    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        self.build_box(tokens, dict)
    }

    fn is_op_boxed(&self, tokens: &VecDeque<Token>) -> bool {
        self.is_op(tokens)
    }
}
