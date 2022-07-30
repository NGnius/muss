use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::Token;
use crate::Context;

use crate::lang::utility::{assert_token, assert_token_raw, assert_type, check_is_type};
use crate::lang::LanguageDictionary;
use crate::lang::{
    BoxedOpFactory, IteratorItem, Op, OpFactory, TypePrimitive, PseudoOp,
};
use crate::lang::{RuntimeError, RuntimeOp, SyntaxError};
use crate::processing::general::Type;

#[derive(Debug)]
pub struct AssignStatement {
    variable_name: String,
    inner_statement: Option<PseudoOp>,
    assign_type: Option<TypePrimitive>,
    context: Option<Context>,
    is_declaration: bool,
    is_simple: bool,
    is_tried: bool,
}

impl Display for AssignStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(inner_statement) = &self.inner_statement {
            write!(f, "{} = {}", self.variable_name, inner_statement)
        } else {
            write!(f, "{} = ???", self.variable_name)
        }
    }
}

impl std::clone::Clone for AssignStatement {
    fn clone(&self) -> Self {
        Self {
            variable_name: self.variable_name.clone(),
            inner_statement: self.inner_statement.clone(),
            assign_type: self.assign_type.clone(),
            context: None,
            is_declaration: self.is_declaration,
            is_simple: self.is_simple,
            is_tried: self.is_tried,
        }
    }
}

impl Iterator for AssignStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_tried {
            return None;
        } else {
            self.is_tried = true;
        }
        if let Some(inner_statement) = &mut self.inner_statement {
            if inner_statement.is_fake() {
                Some(Err(RuntimeError {
                    line: 0,
                    op: (Box::new(self.clone()) as Box<dyn Op>).into(),
                    msg: format!(
                        "Variable {} already assigned, cannot redo assignment",
                        self.variable_name
                    ),
                }))
            } else {
                let mut inner = inner_statement.clone();
                std::mem::swap(inner_statement, &mut inner);
                let real = match inner.unwrap_real() {
                    Ok(real) => real,
                    Err(e) => return Some(Err(e)),
                };
                let result = if self.is_declaration {
                    self
                        .context
                        .as_mut()
                        .unwrap()
                        .variables
                        .declare(&self.variable_name, Type::Op(real))
                } else {
                    self
                        .context
                        .as_mut()
                        .unwrap()
                        .variables
                        .assign(&self.variable_name, Type::Op(real))
                };
                match result {
                    Ok(_) => None,
                    Err(e) => Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
                }
            }
        } else if !self.is_simple {
            panic!(
                "Assignee statement for {} is None but assignment is not simple type",
                self.variable_name
            )
            /*Some(Err(RuntimeError {
                line: 0,
                op: (Box::new(self.clone()) as Box<dyn Op>).into(),
                msg: format!("(BUG) Assignee statement for {} is None but assignment is not simple type", self.variable_name),
            }))*/
        } else {
            let assign_type = self.assign_type.clone().unwrap();
            let result = if self.is_declaration {
                self
                    .context
                    .as_mut()
                    .unwrap()
                    .variables
                    .declare(&self.variable_name, Type::Primitive(assign_type))
            } else {
                self
                    .context
                    .as_mut()
                    .unwrap()
                    .variables
                    .assign(&self.variable_name, Type::Primitive(assign_type))
            };
            match result {
                Ok(_) => None,
                Err(e) => Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(1))
    }
}

impl Op for AssignStatement {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            variable_name: self.variable_name.clone(),
            inner_statement: self
                .inner_statement
                .as_ref()
                .map(|x| PseudoOp::from(x.try_real_ref().unwrap().dup())),
            assign_type: self.assign_type.clone(),
            context: None,
            is_declaration: self.is_declaration,
            is_simple: self.is_simple,
            is_tried: false,
        })
    }
}

pub struct AssignStatementFactory;

impl OpFactory<AssignStatement> for AssignStatementFactory {
    fn is_op(&self, tokens: &VecDeque<Token>) -> bool {
        (tokens.len() >= 3
            &&tokens[0].is_name() // can be any (valid) variable name
            && tokens[1].is_equals())
            || (tokens.len() >= 4
            && tokens[0].is_let()
            && tokens[1].is_name() // any name
            && tokens[2].is_equals())
    }

    fn build_op(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<AssignStatement, SyntaxError> {
        // [let] variable_name = inner_statement
        let is_decl = tokens[0].is_let();
        if is_decl {
            // variable declarations start with let, assignments do not
            assert_token_raw(Token::Let, tokens)?;
        }
        let name = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("variable_name".into()),
            tokens,
        )?;
        assert_token_raw(Token::Equals, tokens)?;
        let is_simple_assign = check_is_type(&tokens[0]);
        if is_simple_assign {
            let simple_type = assert_type(tokens)?;
            Ok(AssignStatement {
                variable_name: name,
                inner_statement: None,
                assign_type: Some(simple_type),
                context: None,
                is_declaration: is_decl,
                is_simple: true,
                is_tried: false,
            })
        } else {
            let inner_statement = dict.try_build_statement(tokens)?;
            Ok(AssignStatement {
                variable_name: name,
                inner_statement: Some(inner_statement.into()),
                assign_type: None,
                context: None,
                is_declaration: is_decl,
                is_simple: false,
                is_tried: false,
            })
        }
    }
}

impl BoxedOpFactory for AssignStatementFactory {
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
