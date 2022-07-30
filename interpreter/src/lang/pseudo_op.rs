#![allow(clippy::borrowed_box)]
use std::fmt::{Debug, Display, Error, Formatter};

use super::Op;
use super::RuntimeError;

/// Mps operation where clones of it emulate the Display behaviour without cloning the data
#[derive(Debug)]
pub enum PseudoOp {
    Real(Box<dyn Op>),
    Fake(String),
}

impl PseudoOp {
    pub fn try_real(&mut self) -> Result<&mut Box<dyn Op>, RuntimeError> {
        match self {
            Self::Real(op) => Ok(op),
            Self::Fake(_) => Err(RuntimeError {
                line: 0,
                op: self.clone(),
                msg: "PseudoOp::Fake is not a real Op".into(),
            }),
        }
    }

    pub fn try_real_ref(&self) -> Result<&Box<dyn Op>, RuntimeError> {
        match self {
            Self::Real(op) => Ok(op),
            Self::Fake(_) => Err(RuntimeError {
                line: 0,
                op: self.clone(),
                msg: "PseudoOp::Fake is not a real Op".into(),
            }),
        }
    }

    pub fn unwrap_real(self) -> Result<Box<dyn Op>, RuntimeError> {
        match self {
            Self::Real(op) => Ok(op),
            Self::Fake(_) => Err(RuntimeError {
                line: 0,
                op: self.clone(),
                msg: "PseudoOp::Fake is not a real Op".into(),
            }),
        }
    }

    #[inline]
    pub fn is_real(&self) -> bool {
        match self {
            Self::Real(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_fake(&self) -> bool {
        match self {
            Self::Fake(_) => true,
            _ => false,
        }
    }

    pub fn from_printable<D: Display + Debug>(item: &D) -> Self {
        Self::Fake(format!("{}", item))
    }
}

impl Clone for PseudoOp {
    fn clone(&self) -> Self {
        match self {
            Self::Real(op) => Self::Fake(format!("{}", op)),
            Self::Fake(s) => Self::Fake(s.clone()),
        }
    }
}

impl Display for PseudoOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Real(op) => write!(f, "{}", op),
            Self::Fake(s) => write!(f, "{}", s),
        }
    }
}

impl std::convert::From<Box<dyn Op>> for PseudoOp {
    fn from(item: Box<dyn Op>) -> Self {
        Self::Real(item)
    }
}
