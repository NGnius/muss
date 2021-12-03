use std::iter::Iterator;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter, Error};

use crate::MpsContext;
use crate::MpsMusicItem;
use crate::tokens::MpsToken;
use super::{MpsOp, MpsOpFactory, BoxedMpsOpFactory};
use super::{SyntaxError, RuntimeError};
use super::utility::{assert_token, assert_token_raw};
use super::db::*;

#[derive(Debug)]
pub struct SqlStatement {
    query: String,
    context: Option<MpsContext>,
    rows: Option<Vec<rusqlite::Result<MpsMusicItem>>>,
    current: usize,
}

impl SqlStatement {
    fn map_item(&mut self, increment: bool) -> Option<Result<MpsMusicItem, RuntimeError>> {
        if let Some(rows) = &self.rows {
            if increment {
                if self.current == rows.len() {
                    return None
                }
                self.current += 1;
            }
            if self.current >= rows.len() {
                None
            } else {
                match &rows[self.current] {
                    Ok(item) => Some(Ok(item.clone())),
                    Err(e) => Some(Err(RuntimeError {
                        line: 0,
                        op: Box::new(self.clone()),
                        msg: format!("SQL music item mapping error: {}", e).into(),
                    }))
                }
            }
        } else {
            Some(Err(RuntimeError {
                line: 0,
                op: Box::new(self.clone()),
                msg: format!("Context error: rows is None").into(),
            }))
        }

    }
}

impl MpsOp for SqlStatement {
    fn enter(&mut self, ctx: MpsContext){
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }
}

impl std::clone::Clone for SqlStatement {
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            context: self.context.clone(),
            rows: None, // TODO use different Result type so this is cloneable
            current: self.current,
        }
    }
}

impl Iterator for SqlStatement {
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rows.is_some() {
            // query has executed, return another result
            self.map_item(true)
        } else {
            let ctx = self.context.as_mut().unwrap();
            // query has not been executed yet
            if let None = ctx.sqlite_connection {
                // connection needs to be created
                match generate_default_db() {
                    Ok(conn) => ctx.sqlite_connection = Some(conn),
                    Err(e) => return Some(Err(RuntimeError{
                        line: 0,
                        op: Box::new(self.clone()),
                        msg: format!("SQL connection error: {}", e).into()
                    }))
                }
            }
            let conn = ctx.sqlite_connection.as_mut().unwrap();
            // execute query
            match perform_query(conn, &self.query) {
                Ok(items) => {
                    self.rows = Some(items);
                    self.map_item(false)
                }
                Err(e) => Some(Err(RuntimeError{
                    line: 0,
                    op: Box::new(self.clone()),
                    msg: e
                }))
            }
        }
    }
}

impl Display for SqlStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "sql(`{}`)", &self.query)
    }
}

pub struct SqlStatementFactory;

impl MpsOpFactory<SqlStatement> for SqlStatementFactory {
    #[inline]
    fn is_op(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() > 3 && tokens[0].is_sql()
    }

    #[inline]
    fn build_op(&self, tokens: &mut VecDeque<MpsToken>) -> Result<SqlStatement, SyntaxError> {
        // sql ( `some query` )
        assert_token_raw(MpsToken::Sql, tokens)?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        let literal = assert_token(|t| {
            match t {
                MpsToken::Literal(query) => Some(query),
                _ => None
            }
        }, MpsToken::Literal("".into()), tokens)?;
        assert_token_raw(MpsToken::CloseBracket, tokens)?;
        Ok(SqlStatement {
            query: literal,
            context: None,
            current: 0,
            rows: None,
        })
    }
}

impl BoxedMpsOpFactory for SqlStatementFactory {
    fn build_op_boxed(&self, tokens: &mut VecDeque<MpsToken>) -> Result<Box<dyn MpsOp>, SyntaxError> {
        self.build_box(tokens)
    }

    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool {
        self.is_op(tokens)
    }
}

fn perform_query(
    conn: &mut rusqlite::Connection,
    query: &str
) -> Result<Vec<rusqlite::Result<MpsMusicItem>>, String> {
    let mut stmt = conn.prepare(query)
        .map_err(|e| format!("SQLite query error: {}", e))?;
    let iter = stmt.query_map([], MpsMusicItem::map_row)
        .map_err(|e| format!("SQLite item mapping error: {}", e))?;
    Ok(iter.collect())
}
