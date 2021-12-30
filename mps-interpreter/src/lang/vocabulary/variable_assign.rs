use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::MpsContext;
use crate::MpsMusicItem;
use crate::tokens::MpsToken;

use crate::lang::{RuntimeError, SyntaxError};
use crate::lang::{MpsOp, PseudoOp, MpsOpFactory, BoxedMpsOpFactory, MpsTypePrimitive};
use crate::lang::MpsLanguageDictionary;
use crate::lang::utility::{assert_token_raw, assert_token, check_is_type, assert_type};
use crate::processing::general::MpsType;

#[derive(Debug)]
pub struct AssignStatement {
    variable_name: String,
    inner_statement: Option<PseudoOp>,
    assign_type: Option<MpsTypePrimitive>,
    context: Option<MpsContext>,
    is_declaration: bool,
    is_simple: bool,
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
        }
    }
}

impl Iterator for AssignStatement {
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner_statement) = &mut self.inner_statement {
            if inner_statement.is_fake() {
                Some(Err(RuntimeError {
                    line: 0,
                    op: (Box::new(self.clone()) as Box<dyn MpsOp>).into(),
                    msg: format!("Variable {} already assigned, cannot redo assignment", self.variable_name),
                }))
            } else {
                let mut inner = inner_statement.clone();
                std::mem::swap(inner_statement, &mut inner);
                let real = match inner.unwrap_real() {
                    Ok(real) => real,
                    Err(e) => return Some(Err(e)),
                };
                let pseudo_clone = self.clone();
                let result;
                if self.is_declaration {
                    result = self.context.as_mut().unwrap()
                        .variables.declare(
                            &self.variable_name,
                            MpsType::Op(real),
                            &mut move ||(Box::new(pseudo_clone.clone()) as Box<dyn MpsOp>).into()
                    );
                } else {
                    result = self.context.as_mut().unwrap()
                        .variables.assign(
                            &self.variable_name,
                            MpsType::Op(real),
                            &mut move ||(Box::new(pseudo_clone.clone()) as Box<dyn MpsOp>).into()
                    );
                }
                match result {
                    Ok(_) => None,
                    Err(e) => Some(Err(e))
                }
            }
        } else if !self.is_simple {
            panic!("Assignee statement for {} is None but assignment is not simple type", self.variable_name)
            /*Some(Err(RuntimeError {
                line: 0,
                op: (Box::new(self.clone()) as Box<dyn MpsOp>).into(),
                msg: format!("(BUG) Assignee statement for {} is None but assignment is not simple type", self.variable_name),
            }))*/
        } else {
            let assign_type = self.assign_type.clone().unwrap();
            let pseudo_clone = self.clone();
            let result;
            if self.is_declaration {
                result = self.context.as_mut().unwrap()
                    .variables.declare(
                        &self.variable_name,
                        MpsType::Primitive(assign_type),
                        &mut move ||(Box::new(pseudo_clone.clone()) as Box<dyn MpsOp>).into()
                );
            } else {
                result = self.context.as_mut().unwrap()
                    .variables.assign(
                        &self.variable_name,
                        MpsType::Primitive(assign_type),
                        &mut move ||(Box::new(pseudo_clone.clone()) as Box<dyn MpsOp>).into()
                );
            }
            match result {
                Ok(_) => None,
                Err(e) => Some(Err(e))
            }
        }
    }
}


impl MpsOp for AssignStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }
}

pub struct AssignStatementFactory;

impl MpsOpFactory<AssignStatement> for AssignStatementFactory {
    fn is_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
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
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<AssignStatement, SyntaxError> {
        // [let] variable_name = inner_statement
        let is_decl = tokens[0].is_let();
        if is_decl { // variable declarations start with let, assignments do not
            assert_token_raw(MpsToken::Let, tokens)?;
        }
        let name = assert_token(|t| match t {
            MpsToken::Name(s) => Some(s),
            _ => None
        }, MpsToken::Name("variable_name".into()), tokens)?;
        assert_token_raw(MpsToken::Equals, tokens)?;
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
            })
        }
    }
}

impl BoxedMpsOpFactory for AssignStatementFactory {
    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsOp>, SyntaxError> {
        self.build_box(tokens, dict)
    }

    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool {
        self.is_op(tokens)
    }
}