//! Basic types for MPS

use std::cmp::{Ord, Ordering};
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum MpsTypePrimitive {
    String(String),
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    Empty,
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
            Self::Bool(x) => format!("{}", x),
            Self::Empty => "".to_owned(),
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
        if let Ok(i) = s.parse::<i64>() {
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

    // math operations

    #[inline]
    pub fn try_add(&self, other: &Self) -> Result<Self, String> {
        match self {
            Self::String(s) => match other {
                Self::String(other_s) => Ok(Self::String(s.to_owned() + other_s)),
                other => Err(format!(
                    "Cannot add {} and {}: incompatible types",
                    self, other
                )),
            },
            Self::Int(i) => match other {
                Self::Int(other_i) => Ok(Self::Int(i + other_i)),
                Self::UInt(u) => Ok(Self::Int(i + *u as i64)),
                Self::Float(f) => Ok(Self::Float(*i as f64 + f)),
                other => Err(format!(
                    "Cannot add {} and {}: incompatible types",
                    self, other
                )),
            },
            Self::UInt(u) => match other {
                Self::UInt(other_u) => Ok(Self::UInt(u + other_u)),
                Self::Int(i) => Ok(Self::UInt(u + *i as u64)),
                Self::Float(f) => Ok(Self::Float(*u as f64 + f)),
                other => Err(format!(
                    "Cannot add {} and {}: incompatible types",
                    self, other
                )),
            },
            Self::Float(f) => match other {
                Self::Float(other_f) => Ok(Self::Float(f + other_f)),
                Self::Int(i) => Ok(Self::Float(f + *i as f64)),
                Self::UInt(u) => Ok(Self::Float(f + *u as f64)),
                other => Err(format!(
                    "Cannot add {} and {}: incompatible types",
                    self, other
                )),
            },
            Self::Bool(_) => Err(format!(
                "Cannot add {} and {}: incompatible types",
                self, other
            )),
            Self::Empty => Err(format!(
                "Cannot add {} and {}: incompatible types",
                self, other
            )),
        }
    }

    #[inline]
    pub fn try_subtract(&self, other: &Self) -> Result<Self, String> {
        match other {
            Self::UInt(other_u) => match self {
                Self::UInt(u) => Ok(Self::UInt(u - other_u)),
                _ => self.try_add(&Self::Int(-(*other_u as i64))),
            },
            other => self.try_add(&other.try_negate()?),
        }
    }

    #[inline]
    pub fn try_negate(&self) -> Result<Self, String> {
        match self {
            Self::Int(i) => Ok(Self::Int(-*i)),
            Self::Float(f) => Ok(Self::Float(-*f)),
            _ => Err(format!("Cannot negate {}: incompatible type", self)),
        }
    }

    #[inline]
    pub fn try_not(&self) -> Result<Self, String> {
        match self {
            Self::Bool(b) => Ok(Self::Bool(!*b)),
            _ => Err(format!(
                "Cannot apply logical NOT to {}: incompatible type",
                self
            )),
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
            Self::Empty => write!(f, "Empty[]"),
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
                Self::Float(f2) => Some(f1.partial_cmp(f2).unwrap_or(std::cmp::Ordering::Less)),
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
            Self::Empty => match other {
                Self::Empty => Some(std::cmp::Ordering::Equal),
                _ => None,
            },
        }
    }
}

impl std::hash::Hash for MpsTypePrimitive {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        match self {
            Self::String(s) => s.hash(state),
            Self::Int(i) => i.hash(state),
            Self::UInt(u) => u.hash(state),
            Self::Float(f_) => (*f_ as u64).hash(state),
            Self::Bool(b) => b.hash(state),
            Self::Empty => {}
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
