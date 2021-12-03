use std::fmt::{Debug, Display, Formatter, Error};

#[derive(Debug, Eq, PartialEq)]
pub enum MpsToken {
    Sql,
    OpenBracket,
    CloseBracket,
    Literal(String),
}

impl MpsToken {
    pub fn parse_from_string(s: String) -> Result<Self, String> {
        match &s as &str {
            "sql" => Ok(Self::Sql),
            "(" => Ok(Self::OpenBracket),
            ")" => Ok(Self::CloseBracket),
            _ => Err(s),
        }
    }

    pub fn is_sql(&self) -> bool {
        match self {
            Self::Sql => true,
            _ => false
        }
    }

    pub fn is_open_bracket(&self) -> bool {
        match self {
            Self::OpenBracket => true,
            _ => false
        }
    }

    pub fn is_close_bracket(&self) -> bool {
        match self {
            Self::CloseBracket => true,
            _ => false
        }
    }

    pub fn is_literal(&self) -> bool {
        match self {
            Self::Literal(_) => true,
            _ => false
        }
    }
}

impl Display for MpsToken {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Sql => write!(f, "sql"),
            Self::OpenBracket => write!(f, "("),
            Self::CloseBracket => write!(f, ")"),
            Self::Literal(s) => write!(f, "\"{}\"", s),
        }
    }
}
