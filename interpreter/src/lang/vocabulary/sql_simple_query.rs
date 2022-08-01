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

#[derive(Debug, Clone)]
enum QueryMode {
    Artist,
    Album,
    Song,
    Genre,
}

impl QueryMode {
    fn from_name(name: String) -> Result<Self, SyntaxError> {
        match &name as &str {
            "artist" => Ok(QueryMode::Artist),
            "album" => Ok(QueryMode::Album),
            "song" => Ok(QueryMode::Song),
            "genre" => Ok(QueryMode::Genre),
            _ => Err(SyntaxError {
                line: 0,
                token: Token::Name("artist|album|song|genre".into()),
                got: Some(Self::tokenify(name)),
            }),
        }
    }

    fn is_valid_name(name: &str) -> bool {
        match name {
            "artist" | "album" | "song" | "genre" => true,
            _ => false,
        }
    }

    #[inline]
    fn tokenify(name: String) -> Token {
        Token::Name(name)
    }

    #[inline]
    fn tokenify_self(&self) -> Token {
        Token::Name(
            match self {
                Self::Artist => "artist",
                Self::Album => "album",
                Self::Song => "song",
                Self::Genre => "genre",
            }
            .into(),
        )
    }
}

#[derive(Debug)]
pub struct SimpleSqlStatement {
    query: String,
    mode: QueryMode,
    context: Option<Context>,
    rows: Option<Box<dyn Op>>,
    is_complete: bool,
}

impl SimpleSqlStatement {
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

impl Op for SimpleSqlStatement {
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
            mode: self.mode.clone(),
            context: None,
            rows: None,
            is_complete: false,
        })
    }
}

impl std::clone::Clone for SimpleSqlStatement {
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            mode: self.mode.clone(),
            context: None, // unecessary to include in clone (not used for displaying)
            rows: None,    // unecessary to include
            is_complete: self.is_complete,
        }
    }
}

impl Iterator for SimpleSqlStatement {
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
            let query_result = match self.mode {
                QueryMode::Artist => ctx.database.artist_like(&self.query),
                QueryMode::Album => ctx.database.album_like(&self.query),
                QueryMode::Song => ctx.database.song_like(&self.query),
                QueryMode::Genre => ctx.database.genre_like(&self.query),
            };
            match query_result {
                Err(e) => {
                    self.is_complete = true;
                    Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))))
                }
                Ok(mut rows) => {
                    //drop(ctx);
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

impl Display for SimpleSqlStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}(`{}`)", self.mode.tokenify_self(), &self.query)
    }
}

pub struct SimpleSqlFunctionFactory;

impl FunctionFactory<SimpleSqlStatement> for SimpleSqlFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        QueryMode::is_valid_name(name)
    }

    fn build_function_params(
        &self,
        mode_name: String,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<SimpleSqlStatement, SyntaxError> {
        // artist|album|song|genre ( `title_like` )
        let literal = assert_token(
            |t| match t {
                Token::Literal(query) => Some(query),
                _ => None,
            },
            Token::Literal("literal".into()),
            tokens,
        )?;
        Ok(SimpleSqlStatement {
            query: literal,
            mode: QueryMode::from_name(mode_name)?,
            context: None,
            is_complete: false,
            rows: None,
        })
    }
}

pub type SimpleSqlStatementFactory =
    FunctionStatementFactory<SimpleSqlStatement, SimpleSqlFunctionFactory>;

#[inline(always)]
pub fn simple_sql_function_factory() -> SimpleSqlStatementFactory {
    SimpleSqlStatementFactory::new(SimpleSqlFunctionFactory)
}
