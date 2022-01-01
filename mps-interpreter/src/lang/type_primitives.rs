//! Basic types for MPS

use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug, Clone)]
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
        let result = match self {
            Self::String(s1) => match other {
                Self::String(s2) => Some(map_ordering(s1.cmp(s2))),
                _ => None,
            },
            Self::Int(i1) => match other {
                Self::Int(i2) => Some(map_ordering(i1.cmp(i2))),
                Self::UInt(i2) => Some(map_ordering((*i1 as i128).cmp(&(*i2 as i128)))),
                Self::Float(i2) => Some(map_ordering(
                    (*i1 as f64)
                        .partial_cmp(&(*i2 as f64))
                        .unwrap_or(std::cmp::Ordering::Less),
                )),
                _ => None,
            },
            Self::UInt(u1) => match other {
                Self::UInt(u2) => Some(map_ordering(u1.cmp(u2))),
                Self::Int(u2) => Some(map_ordering((*u1 as i128).cmp(&(*u2 as i128)))),
                Self::Float(u2) => Some(map_ordering(
                    (*u1 as f64)
                        .partial_cmp(&(*u2 as f64))
                        .unwrap_or(std::cmp::Ordering::Less),
                )),
                _ => None,
            },
            Self::Float(f1) => match other {
                Self::Float(f2) => Some(map_ordering(
                    f1.partial_cmp(f2).unwrap_or(std::cmp::Ordering::Less),
                )),
                Self::Int(f2) => Some(map_ordering(
                    f1.partial_cmp(&(*f2 as f64))
                        .unwrap_or(std::cmp::Ordering::Less),
                )),
                Self::UInt(f2) => Some(map_ordering(
                    f1.partial_cmp(&(*f2 as f64))
                        .unwrap_or(std::cmp::Ordering::Less),
                )),
                _ => None,
            },
            Self::Bool(b1) => match other {
                Self::Bool(b2) => {
                    if *b2 == *b1 {
                        Some(0)
                    } else if *b1 {
                        Some(1)
                    } else {
                        Some(-1)
                    }
                }
                _ => None,
            },
        };
        match result {
            Some(x) => Ok(x),
            None => Err(format!(
                "Cannot compare {} to {}: incompatible types",
                self, other
            )),
        }
    }
}

impl Display for MpsTypePrimitive {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::String(s) => write!(f, "(String) {}", s),
            Self::Int(i) => write!(f, "(Int) {}", *i),
            Self::UInt(u) => write!(f, "(UInt) {}", *u),
            Self::Float(f_) => write!(f, "(Float) {}", *f_),
            Self::Bool(b) => write!(f, "(Bool) {}", *b),
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
