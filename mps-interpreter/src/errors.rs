use std::fmt::{Debug, Display, Error, Formatter};
use std::convert::From;

use crate::lang::{SyntaxError, RuntimeError, MpsLanguageError};
use crate::tokens::ParseError;

#[derive(Debug)]
pub enum MpsError {
    Syntax(SyntaxError),
    Runtime(RuntimeError),
    Parse(ParseError),
}

impl Display for MpsError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Syntax(e) => (e as &dyn Display).fmt(f),
            Self::Runtime(e) => (e as &dyn Display).fmt(f),
            Self::Parse(e) => (e as &dyn Display).fmt(f),
        }
    }
}

impl MpsLanguageError for MpsError {
    fn set_line(&mut self, line: usize) {
        match self {
            Self::Syntax(e) => e.set_line(line),
            Self::Runtime(e) => e.set_line(line),
            Self::Parse(e) => e.set_line(line),
        }
    }
}

impl From<SyntaxError> for MpsError {
    fn from(e: SyntaxError) -> Self {
        Self::Syntax(e)
    }
}

impl From<RuntimeError> for MpsError {
    fn from(e: RuntimeError) -> Self {
        Self::Runtime(e)
    }
}

impl From<ParseError> for MpsError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}
