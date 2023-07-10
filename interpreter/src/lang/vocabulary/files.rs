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
use crate::processing::general::FileIter;

#[derive(Debug)]
pub struct FilesStatement {
    context: Option<Context>,
    // function params
    folder: Option<String>,
    regex: Option<String>,
    recursive: Option<bool>,
    // state
    file_iter: Option<FileIter>,
    has_tried: bool,
}

impl Display for FilesStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "files(")?;
        let mut preceding = false;
        if let Some(folder) = &self.folder {
            write!(f, "folder=`{}`", folder)?;
            preceding = true;
        }
        if let Some(regex) = &self.regex {
            if preceding {
                write!(f, ", ")?;
            } else {
                preceding = true;
            }
            write!(f, "regex=`{}`", regex)?;
        }
        if let Some(recursive) = self.recursive {
            if preceding {
                write!(f, ", ")?;
            }
            write!(f, "recursive={}", recursive)?;
        }
        write!(f, ")")
    }
}

impl std::clone::Clone for FilesStatement {
    fn clone(&self) -> Self {
        Self {
            context: None,
            folder: self.folder.clone(),
            regex: self.regex.clone(),
            recursive: self.recursive,
            file_iter: None,
            has_tried: self.has_tried,
        }
    }
}

impl Iterator for FilesStatement {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.file_iter.is_none() {
            if self.has_tried {
                return None;
            } else {
                self.has_tried = true;
            }
            let iter = self.context.as_mut().unwrap().filesystem.raw(
                self.folder.as_deref(),
                self.regex.as_deref(),
                self.recursive.unwrap_or(true),
            );
            self.file_iter = Some(match iter {
                Ok(x) => x,
                Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            });
        }
        match self.file_iter.as_mut().unwrap().next() {
            Some(Ok(item)) => Some(Ok(item)),
            Some(Err(e)) => Some(Err(RuntimeError {
                line: 0,
                op: PseudoOp::from_printable(self),
                msg: e,
            })),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.file_iter
            .as_ref()
            .map(|x| x.size_hint())
            .unwrap_or((0, None))
    }
}

impl Op for FilesStatement {
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
        self.file_iter = None;
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        let mut clone = self.clone();
        clone.reset().unwrap();
        Box::new(clone)
    }
}

pub struct FilesFunctionFactory;

impl FunctionFactory<FilesStatement> for FilesFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "files"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<FilesStatement, SyntaxError> {
        // files([folder|dir=]"path", [regex|re = "pattern",] [recursive = true|false,])
        let mut root_path = None;
        let mut pattern = None;
        let mut recursive = None;
        if !tokens.is_empty() && !tokens[0].is_close_bracket() {
            if tokens[0].is_literal() {
                // folder is specified without keyword
                root_path = Some(assert_token(
                    |t| match t {
                        Token::Literal(s) => Some(s),
                        _ => None,
                    },
                    Token::Literal("/path/to/music/folder".into()),
                    tokens,
                )?);
                if tokens.len() > 1 && tokens[0].is_comma() {
                    assert_token_raw(Token::Comma, tokens)?;
                }
            }
            // parse keyword function parameters
            let ingest = |tokens2: &mut VecDeque<Token>| {
                if tokens2[0].is_close_bracket() {
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
                        Token::Name(s) => Some(Token::Name(s)),
                        Token::Literal(s) => Some(Token::Literal(s)),
                        _ => None,
                    },
                    Token::Name("value".into()),
                    tokens2,
                )?;
                Ok(Some((param_name, param_val))) // successfully ingested one phrase
            };
            let params = repeated_tokens(ingest, Token::Comma).ingest_all(tokens)?;
            // assign parameters to variables
            for (param, val) in params {
                match &param as &str {
                    "folder" | "dir" => match val {
                        Token::Literal(s) => root_path = Some(s),
                        token => {
                            return Err(SyntaxError {
                                line: 0,
                                token: Token::Literal("/path/to/music/folder".into()),
                                got: Some(token),
                            })
                        }
                    },
                    "regex" | "re" => match val {
                        Token::Literal(s) => pattern = Some(s),
                        token => {
                            return Err(SyntaxError {
                                line: 0,
                                token: Token::Literal("regex pattern".into()),
                                got: Some(token),
                            })
                        }
                    },
                    "recursive" => match val {
                        Token::Name(s) => match &s as &str {
                            "true" => recursive = Some(true),
                            "false" => recursive = Some(false),
                            token => {
                                return Err(SyntaxError {
                                    line: 0,
                                    token: Token::Name("true|false".into()),
                                    got: Some(Token::Name(token.to_owned())),
                                })
                            }
                        },
                        token => {
                            return Err(SyntaxError {
                                line: 0,
                                token: Token::Name("true|false".into()),
                                got: Some(token),
                            })
                        }
                    },
                    s => {
                        return Err(SyntaxError {
                            line: 0,
                            token: Token::Name("folder|regex|recursive".into()),
                            got: Some(Token::Name(s.to_owned())),
                        })
                    }
                }
            }
        }
        Ok(FilesStatement {
            context: None,
            folder: root_path,
            regex: pattern,
            recursive,
            file_iter: None,
            has_tried: false,
        })
    }
}

pub type FilesStatementFactory = FunctionStatementFactory<FilesStatement, FilesFunctionFactory>;

#[inline(always)]
pub fn files_function_factory() -> FilesStatementFactory {
    FilesStatementFactory::new(FilesFunctionFactory)
}
