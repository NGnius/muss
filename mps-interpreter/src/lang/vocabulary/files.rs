use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;

use crate::tokens::MpsToken;
use crate::MpsContext;

use crate::lang::repeated_tokens;
use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{MpsFunctionFactory, MpsFunctionStatementFactory, MpsIteratorItem, MpsOp};
use crate::lang::{RuntimeError, SyntaxError};
use crate::processing::general::FileIter;

#[derive(Debug)]
pub struct FilesStatement {
    context: Option<MpsContext>,
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
            recursive: self.recursive.clone(),
            file_iter: None,
            has_tried: self.has_tried,
        }
    }
}

impl Iterator for FilesStatement {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.file_iter.is_none() {
            if self.has_tried {
                return None;
            } else {
                self.has_tried = true;
            }
            let self_clone = self.clone();
            let iter = self.context.as_mut().unwrap().filesystem.raw(
                self.folder.as_deref(),
                self.regex.as_deref(),
                self.recursive.unwrap_or(true),
                &mut move || (Box::new(self_clone.clone()) as Box<dyn MpsOp>).into(),
            );
            self.file_iter = Some(match iter {
                Ok(x) => x,
                Err(e) => return Some(Err(e)),
            });
        }
        match self.file_iter.as_mut().unwrap().next() {
            Some(Ok(item)) => Some(Ok(item.into())),
            Some(Err(e)) => Some(Err(RuntimeError {
                line: 0,
                op: (Box::new(self.clone()) as Box<dyn MpsOp>).into(),
                msg: e,
            })),
            None => None,
        }
    }
}

impl MpsOp for FilesStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
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
}

pub struct FilesFunctionFactory;

impl MpsFunctionFactory<FilesStatement> for FilesFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "files"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<FilesStatement, SyntaxError> {
        // files([folder|dir=]"path", [regex|re = "pattern"], [recursive = true|false])
        let mut root_path = None;
        let mut pattern = None;
        let mut recursive = None;
        if tokens.len() != 0 {
            if tokens[0].is_literal() {
                // folder is specified without keyword
                root_path = Some(assert_token(
                    |t| match t {
                        MpsToken::Literal(s) => Some(s),
                        _ => None,
                    },
                    MpsToken::Literal("/path/to/music/folder".into()),
                    tokens,
                )?);
                if tokens.len() > 1 && tokens[0].is_comma() {
                    assert_token_raw(MpsToken::Comma, tokens)?;
                }
            }
            // parse keyword function parameters
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
                        MpsToken::Name(s) => Some(MpsToken::Name(s)),
                        MpsToken::Literal(s) => Some(MpsToken::Literal(s)),
                        _ => None,
                    },
                    MpsToken::Name("value".into()),
                    tokens2,
                )?;
                Ok(Some((param_name, param_val))) // successfully ingested one phrase
            };
            let params = repeated_tokens(ingest, MpsToken::Comma).ingest_all(tokens)?;
            // assign parameters to variables
            for (param, val) in params {
                match &param as &str {
                    "folder" | "dir" => match val {
                        MpsToken::Literal(s) => root_path = Some(s),
                        token => {
                            return Err(SyntaxError {
                                line: 0,
                                token: MpsToken::Literal("/path/to/music/folder".into()),
                                got: Some(token),
                            })
                        }
                    },
                    "regex" | "re" => match val {
                        MpsToken::Literal(s) => pattern = Some(s),
                        token => {
                            return Err(SyntaxError {
                                line: 0,
                                token: MpsToken::Literal("regex pattern".into()),
                                got: Some(token),
                            })
                        }
                    },
                    "recursive" => match val {
                        MpsToken::Name(s) => match &s as &str {
                            "true" => recursive = Some(true),
                            "false" => recursive = Some(false),
                            token => {
                                return Err(SyntaxError {
                                    line: 0,
                                    token: MpsToken::Name("true|false".into()),
                                    got: Some(MpsToken::Name(token.to_owned())),
                                })
                            }
                        },
                        token => {
                            return Err(SyntaxError {
                                line: 0,
                                token: MpsToken::Name("true|false".into()),
                                got: Some(token),
                            })
                        }
                    },
                    s => {
                        return Err(SyntaxError {
                            line: 0,
                            token: MpsToken::Name("folder|regex|recursive".into()),
                            got: Some(MpsToken::Name(s.to_owned())),
                        })
                    }
                }
            }
        }
        Ok(FilesStatement {
            context: None,
            folder: root_path,
            regex: pattern,
            recursive: recursive,
            file_iter: None,
            has_tried: false,
        })
    }
}

pub type FilesStatementFactory = MpsFunctionStatementFactory<FilesStatement, FilesFunctionFactory>;

#[inline(always)]
pub fn files_function_factory() -> FilesStatementFactory {
    FilesStatementFactory::new(FilesFunctionFactory)
}
