//! Basic types for MPS

use std::fmt::{Debug, Display, Error, Formatter};
use std::cmp::{Ordering, Ord};

#[derive(Debug, Clone, PartialEq)]
pub enum MpsTypePrimitive {
    String(String),
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
}

impl MpsTypePrimitive {
    #[inline]
    pub fn compare(&self, other: &Self) -> Result<i8, String> {
        let result = self.partial_cmp(other);
        match result {
            Some(x) => Ok(map_ordering(x)),
            None => Err(format!(
                "Cannot compare {} to {}: incompatible types",
                self, other
            )),
        }
    }

    pub fn to_str(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::UInt(x) => format!("{}", x),
            Self::Int(x) => format!("{}", x),
            Self::Float(x) => format!("{}", x),
            Self::Bool(x) => format!("{}", x)
        }
    }

    pub fn to_u64(self) -> Option<u64> {
        match self {
            Self::UInt(x) => Some(x),
            Self::Int(x) => Some(x as _),
            Self::Float(x) => Some(x as _),
            _ => None,
        }
    }

    pub fn to_i64(self) -> Option<i64> {
        match self {
            Self::UInt(x) => Some(x as _),
            Self::Int(x) => Some(x),
            Self::Float(x) => Some(x as _),
            _ => None,
        }
    }

    pub fn parse(s: String) -> Self {
        if  let Ok(i) = s.parse::<i64>() {
            Self::Int(i)
        } else if let Ok(u) = s.parse::<u64>() {
            Self::UInt(u)
        } else if let Ok(f) = s.parse::<f64>() {
            Self::Float(f)
        } else if s == "false" {
            Self::Bool(false)
        } else if s == "true" {
            Self::Bool(true)
        } else {
            Self::String(s)
        }
    }
}

impl Display for MpsTypePrimitive {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::String(s) => write!(f, "String[`{}`]", s),
            Self::Int(i) => write!(f, "Int[{}]", *i),
            Self::UInt(u) => write!(f, "UInt[{}]", *u),
            Self::Float(f_) => write!(f, "Float[{}]", *f_),
            Self::Bool(b) => write!(f, "Bool[{}]", *b),
        }
    }
}

impl PartialOrd for MpsTypePrimitive {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Self::String(s1) => match other {
                Self::String(s2) => Some(s1.cmp(s2)),
                _ => None,
            },
            Self::Int(i1) => match other {
                Self::Int(i2) => Some(i1.cmp(i2)),
                Self::UInt(i2) => Some((*i1 as i128).cmp(&(*i2 as i128))),
                Self::Float(i2) => Some(
                    (*i1 as f64)
                        .partial_cmp(&(*i2 as f64))
                        .unwrap_or(std::cmp::Ordering::Less),
                ),
                _ => None,
            },
            Self::UInt(u1) => match other {
                Self::UInt(u2) => Some(u1.cmp(u2)),
                Self::Int(u2) => Some((*u1 as i128).cmp(&(*u2 as i128))),
                Self::Float(u2) => Some(
                    (*u1 as f64)
                        .partial_cmp(&(*u2 as f64))
                        .unwrap_or(std::cmp::Ordering::Less),
                ),
                _ => None,
            },
            Self::Float(f1) => match other {
                Self::Float(f2) => Some(
                    f1.partial_cmp(f2).unwrap_or(std::cmp::Ordering::Less),
                ),
                Self::Int(f2) => Some(
                    f1.partial_cmp(&(*f2 as f64))
                        .unwrap_or(std::cmp::Ordering::Less),
                ),
                Self::UInt(f2) => Some(
                    f1.partial_cmp(&(*f2 as f64))
                        .unwrap_or(std::cmp::Ordering::Less),
                ),
                _ => None,
            },
            Self::Bool(b1) => match other {
                Self::Bool(b2) => {
                    if *b2 == *b1 {
                        Some(std::cmp::Ordering::Equal)
                    } else if *b1 {
                        Some(std::cmp::Ordering::Greater)
                    } else {
                        Some(std::cmp::Ordering::Less)
                    }
                }
                _ => None,
            },
        }
    }
}

#[inline]
fn map_ordering(ordering: std::cmp::Ordering) -> i8 {
    match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

impl std::convert::From<String> for MpsTypePrimitive {
    fn from(item: String) -> Self {
        Self::String(item)
    }
}

impl std::convert::From<i64> for MpsTypePrimitive {
    fn from(item: i64) -> Self {
        Self::Int(item)
    }
}

impl std::convert::From<u64> for MpsTypePrimitive {
    fn from(item: u64) -> Self {
        Self::UInt(item)
    }
}

impl std::convert::From<f64> for MpsTypePrimitive {
    fn from(item: f64) -> Self {
        Self::Float(item)
    }
}

impl std::convert::From<bool> for MpsTypePrimitive {
    fn from(item: bool) -> Self {
        Self::Bool(item)
    }
}
