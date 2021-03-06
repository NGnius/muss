use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::utility::assert_token;
use crate::lang::{
    FunctionFactory, FunctionStatementFactory, IteratorItem, LanguageDictionary, Op, PseudoOp,
};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
use crate::tokens::Token;
use crate::Context;
use crate::Item;

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
    rows: Option<Vec<Result<Item, RuntimeMsg>>>,
    current: usize,
}

impl SimpleSqlStatement {
    fn get_item(&mut self, increment: bool) -> Option<IteratorItem> {
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

impl Op for SimpleSqlStatement {
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
        self.rows = None;
        self.current = 0;
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            query: self.query.clone(),
            mode: self.mode.clone(),
            context: None,
            rows: None,
            current: 0,
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
            current: self.current,
        }
    }
}

impl Iterator for SimpleSqlStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rows.is_some() {
            // query has executed, return another result
            self.get_item(true)
        } else {
            let fake = PseudoOp::from_printable(self);
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
            current: 0,
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
