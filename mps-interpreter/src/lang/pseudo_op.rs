use std::fmt::{Debug, Display, Error, Formatter};

use super::MpsOp;
use super::RuntimeError;

/// Mps operation where clones of it emulate the Display behaviour without cloning the data
#[derive(Debug)]
pub enum PseudoOp {
    Real(Box<dyn MpsOp>),
    Fake(String),
}

impl PseudoOp {
    pub fn try_real(&mut self) -> Result<&mut Box<dyn MpsOp>, RuntimeError> {
        match self {
            Self::Real(op) => Ok(op),
            Self::Fake(_) => Err(RuntimeError {
                line: 0,
                op: self.clone(),
                msg: "PseudoOp::Fake is not a real MpsOp".into(),
            }),
        }
    }

    pub fn try_real_ref(&self) -> Result<&Box<dyn MpsOp>, RuntimeError> {
        match self {
            Self::Real(op) => Ok(op),
            Self::Fake(_) => Err(RuntimeError {
                line: 0,
                op: self.clone(),
                msg: "PseudoOp::Fake is not a real MpsOp".into(),
            }),
        }
    }

    pub fn unwrap_real(self) -> Result<Box<dyn MpsOp>, RuntimeError> {
        match self {
            Self::Real(op) => {
                let result = Ok(op);
                result
            }
            Self::Fake(_) => Err(RuntimeError {
                line: 0,
                op: self.clone(),
                msg: "PseudoOp::Fake is not a real MpsOp".into(),
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

impl std::convert::From<Box<dyn MpsOp>> for PseudoOp {
    fn from(item: Box<dyn MpsOp>) -> Self {
        Self::Real(item)
    }
}
