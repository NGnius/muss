use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::Token;
use crate::Context;

use crate::lang::repeated_tokens;
use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::LanguageDictionary;
use crate::lang::{FunctionFactory, FunctionStatementFactory, IteratorItem, Op};
use crate::lang::{PseudoOp, RuntimeError, RuntimeOp, SyntaxError};

#[derive(Debug)]
pub struct SqlInitStatement {
    context: Option<Context>,
    params: HashMap<String, String>,
    has_tried: bool,
}

impl Display for SqlInitStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "sql_init(")?;
        for (key, val) in self.params.iter() {
            write!(f, "{} = {},", key, val)?;
        }
        write!(f, ")")
    }
}

impl std::clone::Clone for SqlInitStatement {
    fn clone(&self) -> Self {
        Self {
            context: None,
            params: HashMap::new(),
            has_tried: self.has_tried,
        }
    }
}

impl Iterator for SqlInitStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_tried {
            return None;
        }
        // execute
        match self
            .context
            .as_mut()
            .unwrap()
            .database
            .init_with_params(&self.params)
        {
            Ok(_) => None,
            Err(e) => Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl Op for SqlInitStatement {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.has_tried = false;
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            context: None,
            params: self.params.clone(),
            has_tried: false,
        })
    }
}

pub struct SqlInitFunctionFactory;

impl FunctionFactory<SqlInitStatement> for SqlInitFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "sql_init"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<SqlInitStatement, SyntaxError> {
        let ingest = |tokens2: &mut VecDeque<Token>| {
            if tokens2.len() < 3 {
                return Ok(None);
            } // nothing wrong, nothing left to ingest
            let param_name = assert_token(
                |t| match t {
                    Token::Name(s) => Some(s),
                    _ => None,
                },
                Token::Name("param".into()),
                tokens2,
            )?;
            assert_token_raw(Token::Equals, tokens2)?;
            let param_val = assert_token(
                |t| match t {
                    Token::Name(s) => Some(s),
                    Token::Literal(s) => Some(s),
                    _ => None,
                },
                Token::Name("value".into()),
                tokens2,
            )?;
            Ok(Some((param_name, param_val))) // successfully ingested one phrase
        };
        let params = repeated_tokens(ingest, Token::Comma).ingest_all(tokens)?;
        Ok(SqlInitStatement {
            context: None,
            params: HashMap::from_iter(params),
            has_tried: false,
        })
    }
}

pub type SqlInitStatementFactory =
    FunctionStatementFactory<SqlInitStatement, SqlInitFunctionFactory>;

#[inline(always)]
pub fn sql_init_function_factory() -> SqlInitStatementFactory {
    SqlInitStatementFactory::new(SqlInitFunctionFactory)
}
