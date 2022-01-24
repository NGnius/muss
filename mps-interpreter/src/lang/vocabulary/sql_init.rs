use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::MpsToken;
use crate::MpsContext;

use crate::lang::repeated_tokens;
use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::MpsLanguageDictionary;
use crate::lang::SyntaxError;
use crate::lang::{MpsFunctionFactory, MpsFunctionStatementFactory, MpsIteratorItem, MpsOp};

#[derive(Debug)]
pub struct SqlInitStatement {
    context: Option<MpsContext>,
    params: HashMap<String, String>,
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
        }
    }
}

impl Iterator for SqlInitStatement {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let pseudo_clone = self.clone();
        // execute
        match self
            .context
            .as_mut()
            .unwrap()
            .database
            .init_with_params(&self.params, &mut move || {
                (Box::new(pseudo_clone.clone()) as Box<dyn MpsOp>).into()
            }) {
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl MpsOp for SqlInitStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }
}

pub struct SqlInitFunctionFactory;

impl MpsFunctionFactory<SqlInitStatement> for SqlInitFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "sql_init"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<SqlInitStatement, SyntaxError> {
        let ingest = |tokens2: &mut VecDeque<MpsToken>| {
            if tokens2.len() < 3 {
                return Ok(None);
            } // nothing wrong, nothing left to ingest
            let param_name = assert_token(
                |t| match t {
                    MpsToken::Name(s) => Some(s),
                    _ => None,
                },
                MpsToken::Name("param".into()),
                tokens2,
            )?;
            assert_token_raw(MpsToken::Equals, tokens2)?;
            let param_val = assert_token(
                |t| match t {
                    MpsToken::Name(s) => Some(s),
                    MpsToken::Literal(s) => Some(s),
                    _ => None,
                },
                MpsToken::Name("value".into()),
                tokens2,
            )?;
            Ok(Some((param_name, param_val))) // successfully ingested one phrase
        };
        let params = repeated_tokens(ingest, MpsToken::Comma).ingest_all(tokens)?;
        Ok(SqlInitStatement {
            context: None,
            params: HashMap::from_iter(params),
        })
    }
}

pub type SqlInitStatementFactory =
    MpsFunctionStatementFactory<SqlInitStatement, SqlInitFunctionFactory>;

#[inline(always)]
pub fn sql_init_function_factory() -> SqlInitStatementFactory {
    SqlInitStatementFactory::new(SqlInitFunctionFactory)
}
