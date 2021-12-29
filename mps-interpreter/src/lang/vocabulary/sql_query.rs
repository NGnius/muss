use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::utility::{assert_token, assert_token_raw, check_name, assert_name};
use crate::lang::{BoxedMpsOpFactory, MpsLanguageDictionary, MpsOp, MpsOpFactory};
use crate::lang::{RuntimeError, SyntaxError};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsMusicItem;
//use super::db::*;

#[derive(Debug)]
pub struct SqlStatement {
    query: String,
    context: Option<MpsContext>,
    rows: Option<Vec<Result<MpsMusicItem, RuntimeError>>>,
    current: usize,
}

impl SqlStatement {
    fn get_item(&mut self, increment: bool) -> Option<Result<MpsMusicItem, RuntimeError>> {
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
                match &rows[self.current] {
                    Ok(item) => Some(Ok(item.clone())),
                    Err(e) => Some(Err(RuntimeError {
                        line: e.line,
                        op: (Box::new(self.clone()) as Box<dyn MpsOp>).into(),
                        msg: e.msg.clone(),
                    })),
                }
            }
        } else {
            Some(Err(RuntimeError {
                line: 0,
                op: (Box::new(self.clone()) as Box<dyn MpsOp>).into(),
                msg: format!("Context error: rows is None").into(),
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
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rows.is_some() {
            // query has executed, return another result
            self.get_item(true)
        } else {
            let self_clone = self.clone();
            let ctx = self.context.as_mut().unwrap();
            // query has not been executed yet
            match ctx
                .database
                .raw(&self.query, &mut move || (Box::new(self_clone.clone()) as Box<dyn MpsOp>).into())
            {
                Err(e) => return Some(Err(e)),
                Ok(rows) => {
                    self.rows = Some(rows);
                    self.get_item(false)
                }
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
        tokens.len() > 3
            && check_name("sql", &tokens[0])
            && tokens[1].is_open_bracket()
            && tokens[2].is_literal()
            && tokens[3].is_close_bracket()
    }

    #[inline]
    fn build_op(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<SqlStatement, SyntaxError> {
        // sql ( `some query` )
        assert_name("sql", tokens)?;
        //assert_token_raw(MpsToken::Sql, tokens)?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        let literal = assert_token(
            |t| match t {
                MpsToken::Literal(query) => Some(query),
                _ => None,
            },
            MpsToken::Literal("".into()),
            tokens,
        )?;
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
