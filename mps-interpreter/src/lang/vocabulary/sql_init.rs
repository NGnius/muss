use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::collections::HashMap;

use crate::MpsContext;
use crate::MpsMusicItem;
use crate::tokens::MpsToken;

use crate::lang::{RuntimeError, SyntaxError};
use crate::lang::{MpsOp, SimpleMpsOpFactory, MpsOpFactory, BoxedMpsOpFactory};
use crate::lang::MpsLanguageDictionary;
use crate::lang::utility::{assert_token_raw, check_name, assert_name, assert_token};
use crate::lang::repeated_tokens;

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
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let pseudo_clone = self.clone();
        // execute
        match self.context.as_mut().unwrap().database.init_with_params(&self.params,
            &mut move || (Box::new(pseudo_clone.clone()) as Box<dyn MpsOp>).into()) {
            Ok(_) => None,
            Err(e) => Some(Err(e))
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

pub struct SqlInitStatementFactory;

impl SimpleMpsOpFactory<SqlInitStatement> for SqlInitStatementFactory {
    fn is_op_simple(&self, tokens: &VecDeque<MpsToken>) -> bool {
        tokens.len() >= 3
        && check_name("sql_init", &tokens[0])
        && tokens[1].is_open_bracket()
    }

    fn build_op_simple(
        &self,
        tokens: &mut VecDeque<MpsToken>,
    ) -> Result<SqlInitStatement, SyntaxError> {
        assert_name("sql_init", tokens)?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        let ingest = |tokens2: &mut VecDeque<MpsToken>| {
            if tokens2.len() < 3 {return Ok(None);} // nothing wrong, nothing left to ingest
            let param_name = assert_token(|t| match t {
                MpsToken::Name(s) => Some(s),
                _ => None,
            }, MpsToken::Name("param".into()), tokens2)?;
            assert_token_raw(MpsToken::Equals, tokens2)?;
            let param_val = assert_token(|t| match t {
                MpsToken::Name(s) => Some(s),
                MpsToken::Literal(s) => Some(s),
                _ => None,
            }, MpsToken::Name("value".into()), tokens2)?;
            Ok(Some((param_name, param_val))) // successfully ingested one phrase
        };
        let params = repeated_tokens(ingest, MpsToken::Comma).ingest_all(tokens)?;
        assert_token_raw(MpsToken::CloseBracket, tokens)?;
        Ok(SqlInitStatement {
            context: None,
            params: HashMap::from_iter(params),
        })
    }
}

impl BoxedMpsOpFactory for SqlInitStatementFactory {
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
