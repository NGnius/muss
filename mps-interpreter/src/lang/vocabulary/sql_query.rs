use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::utility::assert_token;
use crate::lang::{
    MpsFunctionFactory, MpsFunctionStatementFactory, MpsIteratorItem, MpsLanguageDictionary, MpsOp,
    PseudoOp,
};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsItem;
//use super::db::*;

#[derive(Debug)]
pub struct SqlStatement {
    query: String,
    context: Option<MpsContext>,
    rows: Option<Vec<Result<MpsItem, RuntimeMsg>>>,
    current: usize,
}

impl SqlStatement {
    fn get_item(&mut self, increment: bool) -> Option<MpsIteratorItem> {
        let fake = PseudoOp::from_printable(self);
        if let Some(rows) = &self.rows {
            if increment {
                if self.current == rows.len() {
                    return None;
                }
                self.current += 1;
            }
            if self.current >= rows.len() {
                None
            } else {
                //Some(rows[self.current].clone())
                match rows[self.current].clone() {
                    Ok(item) => Some(Ok(item)),
                    Err(e) => Some(Err(e.with(RuntimeOp(fake)))),
                }
            }
        } else {
            Some(Err(RuntimeError {
                line: 0,
                op: fake,
                msg: "Context error: rows is None".to_string(),
            }))
        }
    }
}

impl MpsOp for SqlStatement {
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
        self.rows = None;
        self.current = 0;
        Ok(())
    }

    fn dup(&self) -> Box<dyn MpsOp> {
        Box::new(Self {
            query: self.query.clone(),
            context: None,
            rows: None,
            current: 0,
        })
    }
}

impl std::clone::Clone for SqlStatement {
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            context: None, // unecessary to include in clone (not used for displaying)
            rows: None,    // unecessary to include
            current: self.current,
        }
    }
}

impl Iterator for SqlStatement {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rows.is_some() {
            // query has executed, return another result
            self.get_item(true)
        } else {
            let fake = PseudoOp::from_printable(self);
            let ctx = self.context.as_mut().unwrap();
            // query has not been executed yet
            match ctx.database.raw(&self.query) {
                Err(e) => {
                    self.rows = Some(Vec::with_capacity(0));
                    Some(Err(e.with(RuntimeOp(fake))))
                }
                Ok(rows) => {
                    self.rows = Some(rows);
                    self.get_item(false)
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.rows.as_ref().map(|x| x.len());
        (len.unwrap_or(0), len)
    }
}

impl Display for SqlStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "sql(`{}`)", &self.query)
    }
}

pub struct SqlFunctionFactory;

impl MpsFunctionFactory<SqlStatement> for SqlFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "sql"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<SqlStatement, SyntaxError> {
        // sql ( `some query` )
        let literal = assert_token(
            |t| match t {
                MpsToken::Literal(query) => Some(query),
                _ => None,
            },
            MpsToken::Literal("".into()),
            tokens,
        )?;
        Ok(SqlStatement {
            query: literal,
            context: None,
            current: 0,
            rows: None,
        })
    }
}

pub type SqlStatementFactory = MpsFunctionStatementFactory<SqlStatement, SqlFunctionFactory>;

#[inline(always)]
pub fn sql_function_factory() -> SqlStatementFactory {
    SqlStatementFactory::new(SqlFunctionFactory)
}
