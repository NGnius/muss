use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::utility::assert_token;
use crate::lang::{
    FunctionFactory, FunctionStatementFactory, IteratorItem, LanguageDictionary, Op, PseudoOp,
};
use crate::lang::{RuntimeError, RuntimeOp, SyntaxError};
use crate::tokens::Token;
use crate::Context;
//use super::db::*;

#[derive(Debug)]
pub struct SqlStatement {
    query: String,
    context: Option<Context>,
    rows: Option<Box<dyn Op>>,
    is_complete: bool,
}

impl SqlStatement {
    fn get_item(&mut self) -> Option<IteratorItem> {
        let result = self.rows.as_mut().unwrap().next().map(|opt| opt.map_err(|mut e| {
            e.op = PseudoOp::from_printable(self);
            e
        }));
        if result.is_none() {
            self.is_complete = true;
        }
        result
    }
}

impl Op for SqlStatement {
    fn enter(&mut self, ctx: Context) {
        if let Some(rows) = &mut self.rows {
            rows.enter(ctx);
        } else {
            self.context = Some(ctx);
        }
    }

    fn escape(&mut self) -> Context {
        if self.context.is_some() {
            self.context.take().unwrap()
        } else {
            self.rows.as_mut().unwrap().escape()
        }

    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        if let Some(mut rows) = self.rows.take() {
            self.context = Some(rows.escape());
        }
        self.is_complete = false;
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            query: self.query.clone(),
            context: None,
            rows: None,
            is_complete: false,
        })
    }
}

impl std::clone::Clone for SqlStatement {
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            context: None, // unecessary to include in clone (not used for displaying)
            rows: None,    // unecessary to include
            is_complete: self.is_complete,
        }
    }
}

impl Iterator for SqlStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_complete {
            return None;
        }
        if self.rows.is_some() {
            // query has executed, return another result
            self.get_item()
        } else {
            let ctx = self.context.as_mut().unwrap();
            // query has not been executed yet
            match ctx.database.raw(&self.query) {
                Err(e) => {
                    self.is_complete = true;
                    Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))))
                }
                Ok(mut rows) => {
                    rows.enter(self.context.take().unwrap());
                    self.rows = Some(rows);
                    self.get_item()
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rows.as_ref().map(|x| x.size_hint()).unwrap_or_default()
    }
}

impl Display for SqlStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "sql(`{}`)", &self.query)
    }
}

pub struct SqlFunctionFactory;

impl FunctionFactory<SqlStatement> for SqlFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "sql"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<SqlStatement, SyntaxError> {
        // sql ( `some query` )
        let literal = assert_token(
            |t| match t {
                Token::Literal(query) => Some(query),
                _ => None,
            },
            Token::Literal("".into()),
            tokens,
        )?;
        Ok(SqlStatement {
            query: literal,
            context: None,
            rows: None,
            is_complete: false,
        })
    }
}

pub type SqlStatementFactory = FunctionStatementFactory<SqlStatement, SqlFunctionFactory>;

#[inline(always)]
pub fn sql_function_factory() -> SqlStatementFactory {
    SqlStatementFactory::new(SqlFunctionFactory)
}
