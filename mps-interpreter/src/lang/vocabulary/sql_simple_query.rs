use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::lang::utility::assert_token;
use crate::lang::{MpsFunctionFactory, MpsFunctionStatementFactory, MpsLanguageDictionary, MpsOp};
use crate::lang::{RuntimeError, SyntaxError};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsMusicItem;

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
                token: MpsToken::Name("artist|album|song|genre".into()),
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
    fn tokenify(name: String) -> MpsToken {
        MpsToken::Name(name)
    }

    #[inline]
    fn tokenify_self(&self) -> MpsToken {
        MpsToken::Name(
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
    context: Option<MpsContext>,
    rows: Option<Vec<Result<MpsMusicItem, RuntimeError>>>,
    current: usize,
}

impl SimpleSqlStatement {
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

impl MpsOp for SimpleSqlStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
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
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rows.is_some() {
            // query has executed, return another result
            self.get_item(true)
        } else {
            let self_clone = self.clone();
            let ctx = self.context.as_mut().unwrap();
            // query has not been executed yet
            let query_result = match self.mode {
                QueryMode::Artist => ctx.database.artist_like(&self.query, &mut move || {
                    (Box::new(self_clone.clone()) as Box<dyn MpsOp>).into()
                }),
                QueryMode::Album => ctx.database.album_like(&self.query, &mut move || {
                    (Box::new(self_clone.clone()) as Box<dyn MpsOp>).into()
                }),
                QueryMode::Song => ctx.database.song_like(&self.query, &mut move || {
                    (Box::new(self_clone.clone()) as Box<dyn MpsOp>).into()
                }),
                QueryMode::Genre => ctx.database.genre_like(&self.query, &mut move || {
                    (Box::new(self_clone.clone()) as Box<dyn MpsOp>).into()
                }),
            };
            match query_result {
                Err(e) => {
                    self.rows = Some(Vec::with_capacity(0));
                    return Some(Err(e));
                },
                Ok(rows) => {
                    self.rows = Some(rows);
                    self.get_item(false)
                }
            }
        }
    }
}

impl Display for SimpleSqlStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}(`{}`)", self.mode.tokenify_self(), &self.query)
    }
}

pub struct SimpleSqlFunctionFactory;

impl MpsFunctionFactory<SimpleSqlStatement> for SimpleSqlFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        QueryMode::is_valid_name(name)
    }

    fn build_function_params(
        &self,
        mode_name: String,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<SimpleSqlStatement, SyntaxError> {
        // artist|album|song|genre ( `title_like` )
        let literal = assert_token(
            |t| match t {
                MpsToken::Literal(query) => Some(query),
                _ => None,
            },
            MpsToken::Literal("literal".into()),
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
    MpsFunctionStatementFactory<SimpleSqlStatement, SimpleSqlFunctionFactory>;

#[inline(always)]
pub fn simple_sql_function_factory() -> SimpleSqlStatementFactory {
    SimpleSqlStatementFactory::new(SimpleSqlFunctionFactory)
}
