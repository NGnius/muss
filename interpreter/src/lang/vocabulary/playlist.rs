use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::Token;
use crate::Context;

use crate::lang::LanguageDictionary;
use crate::lang::{FunctionFactory, FunctionStatementFactory, IteratorItem, Op, GeneratorOp};
use crate::lang::{PseudoOp, RuntimeError, RuntimeOp, SyntaxError, Lookup, TypePrimitive};
//use crate::processing::general::FileIter;
use crate::processing::general::Type;

#[derive(Debug)]
pub struct PlaylistStatement {
    context: Option<Context>,
    // function params
    file: Lookup,
    // state
    playlist_iter: Option<GeneratorOp>,
    has_tried: bool,
}

impl Display for PlaylistStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "playlist({})", self.file)
    }
}

impl std::clone::Clone for PlaylistStatement {
    fn clone(&self) -> Self {
        Self {
            context: None,
            file: self.file.clone(),
            playlist_iter: None,
            has_tried: self.has_tried,
        }
    }
}

impl Iterator for PlaylistStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.playlist_iter.is_none() {
            if self.has_tried {
                return None;
            } else {
                self.has_tried = true;
            }
            let ctx = self.context.as_mut().unwrap();
            let file = match self.file.get(ctx) {
                Ok(Type::Primitive(TypePrimitive::String(s))) => s.to_owned(),
                Ok(x) => return Some(Err(
                    RuntimeError {
                        msg: format!("Cannot use {} as filepath", x),
                        line: 0,
                        op: PseudoOp::from_printable(self),
                    }
                )),
                Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            };
            let iter = ctx.filesystem.read_file(&file);
            self.playlist_iter = Some(match iter {
                Ok(mut x) => {
                    x.enter(self.context.take().unwrap());
                    x
                },
                Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            });
        }
        match self.playlist_iter.as_mut().unwrap().next() {
            Some(Ok(item)) => Some(Ok(item)),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.playlist_iter
            .as_ref()
            .map(|x| x.size_hint())
            .unwrap_or((0, None))
    }
}

impl Op for PlaylistStatement {
    fn enter(&mut self, ctx: Context) {
        if let Some(playlist) = &mut self.playlist_iter {
            playlist.enter(ctx);
        } else {
            self.context = Some(ctx)
        }
    }

    fn escape(&mut self) -> Context {
        if let Some(playlist) = &mut self.playlist_iter {
            playlist.escape()
        } else {
            self.context.take().unwrap()
        }
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.has_tried = false;
        if let Some(playlist) = &mut self.playlist_iter {
            self.context = Some(playlist.escape());
        }
        self.playlist_iter = None;
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        let mut clone = self.clone();
        clone.reset().unwrap();
        Box::new(clone)
    }
}

pub struct PlaylistFunctionFactory;

impl FunctionFactory<PlaylistStatement> for PlaylistFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "playlist"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<PlaylistStatement, SyntaxError> {
        // playlist(filepath)
        let filepath_lookup = Lookup::parse(tokens)?;
        Ok(PlaylistStatement {
            context: None,
            file: filepath_lookup,
            playlist_iter: None,
            has_tried: false,
        })
    }
}

pub type PlaylistStatementFactory = FunctionStatementFactory<PlaylistStatement, PlaylistFunctionFactory>;

#[inline(always)]
pub fn playlist_function_factory() -> PlaylistStatementFactory {
    PlaylistStatementFactory::new(PlaylistFunctionFactory)
}
