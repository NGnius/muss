use std::fmt::{Debug, Display, Error, Formatter};

use super::PseudoOp;
use crate::tokens::Token;

#[derive(Debug)]
pub struct SyntaxError {
    pub line: usize,
    pub token: Token,
    pub got: Option<Token>,
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match &self.got {
            Some(t) => write!(
                f,
                "SyntaxError (line {}): Expected {}, got {}",
                &self.line, &self.token, t
            ),
            None => write!(
                f,
                "SyntaxError (line {}): Expected {}, got nothing",
                &self.line, &self.token
            ),
        }
    }
}

impl LanguageError for SyntaxError {
    fn set_line(&mut self, line: usize) {
        self.line = line
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub line: usize,
    pub op: PseudoOp,
    pub msg: String,
}

impl RuntimeError {
    pub fn decompose(self) -> (RuntimeOp, RuntimeMsg) {
        (RuntimeOp(self.op), RuntimeMsg(self.msg))
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} (line {}): {}", &self.msg, &self.line, &self.op)
    }
}

impl std::hash::Hash for RuntimeError {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.line.hash(state);
        self.msg.hash(state);
    }
}

impl std::cmp::PartialEq for RuntimeError {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.msg == other.msg
    }
}

impl std::cmp::Eq for RuntimeError {}

impl LanguageError for RuntimeError {
    fn set_line(&mut self, line: usize) {
        self.line = line
    }
}

pub trait LanguageError: Display + Debug {
    fn set_line(&mut self, line: usize);
}

// RuntimeError builder components
#[derive(Debug, Clone, Hash)]
pub struct RuntimeMsg(pub String);

impl RuntimeMsg {
    pub fn with(self, op: RuntimeOp) -> RuntimeError {
        RuntimeError {
            line: 0,
            op: op.0,
            msg: self.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeOp(pub PseudoOp);

impl RuntimeOp {
    pub fn with(self, msg: RuntimeMsg) -> RuntimeError {
        RuntimeError {
            line: 0,
            op: self.0,
            msg: msg.0,
        }
    }
}
