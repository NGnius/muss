use std::fmt::{Debug, Display, Error, Formatter};

use super::PseudoOp;
use crate::tokens::MpsToken;

#[derive(Debug)]
pub struct SyntaxError {
    pub line: usize,
    pub token: MpsToken,
    pub got: Option<MpsToken>,
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

impl MpsLanguageError for SyntaxError {
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

impl MpsLanguageError for RuntimeError {
    fn set_line(&mut self, line: usize) {
        self.line = line
    }
}

pub trait MpsLanguageError: Display + Debug {
    fn set_line(&mut self, line: usize);
}

// RuntimeError builder components
#[derive(Debug, Clone)]
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
