use std::convert::From;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::{LanguageError, RuntimeError, SyntaxError};
use crate::tokens::ParseError;

#[derive(Debug)]
pub enum InterpreterError {
    Syntax(SyntaxError),
    Runtime(RuntimeError),
    Parse(ParseError),
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Syntax(e) => (e as &dyn Display).fmt(f),
            Self::Runtime(e) => (e as &dyn Display).fmt(f),
            Self::Parse(e) => (e as &dyn Display).fmt(f),
        }
    }
}

impl LanguageError for InterpreterError {
    fn set_line(&mut self, line: usize) {
        match self {
            Self::Syntax(e) => e.set_line(line),
            Self::Runtime(e) => e.set_line(line),
            Self::Parse(e) => e.set_line(line),
        }
    }
}

impl From<SyntaxError> for InterpreterError {
    fn from(e: SyntaxError) -> Self {
        Self::Syntax(e)
    }
}

impl From<RuntimeError> for InterpreterError {
    fn from(e: RuntimeError) -> Self {
        Self::Runtime(e)
    }
}

impl From<ParseError> for InterpreterError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}
