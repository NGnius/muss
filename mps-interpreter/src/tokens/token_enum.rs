use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MpsToken {
    //Sql,
    OpenBracket,
    CloseBracket,
    Comma,
    Literal(String),
    Name(String),
    //Octothorpe,
    Comment(String),
    Equals,
    Let,
    OpenAngleBracket,
    CloseAngleBracket,
    Dot,
    Exclamation,
    Interrogation,
}

impl MpsToken {
    pub fn parse_from_string(s: String) -> Result<Self, String> {
        match &s as &str {
            //"sql" => Ok(Self::Sql),
            "(" => Ok(Self::OpenBracket),
            ")" => Ok(Self::CloseBracket),
            "," => Ok(Self::Comma),
            //"#" => Ok(Self::Octothorpe),
            "=" => Ok(Self::Equals),
            "let" => Ok(Self::Let),
            "<" => Ok(Self::OpenAngleBracket),
            ">" => Ok(Self::CloseAngleBracket),
            "." => Ok(Self::Dot),
            "!" => Ok(Self::Exclamation),
            "?" => Ok(Self::Interrogation),
            _ => {
                // name validation
                let mut ok = true;
                for invalid_c in ["-", "+", ",", " ", "/", "\n", "\r", "!", "?", "=", "."] {
                    if s.contains(invalid_c) {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    Ok(Self::Name(s))
                } else {
                    Err(s)
                }
            }
        }
    }

    /*pub fn is_sql(&self) -> bool {
        match self {
            Self::Sql => true,
            _ => false,
        }
    }*/

    pub fn is_open_bracket(&self) -> bool {
        match self {
            Self::OpenBracket => true,
            _ => false,
        }
    }

    pub fn is_close_bracket(&self) -> bool {
        match self {
            Self::CloseBracket => true,
            _ => false,
        }
    }

    pub fn is_comma(&self) -> bool {
        match self {
            Self::Comma => true,
            _ => false,
        }
    }

    pub fn is_literal(&self) -> bool {
        match self {
            Self::Literal(_) => true,
            _ => false,
        }
    }

    pub fn is_name(&self) -> bool {
        match self {
            Self::Name(_) => true,
            _ => false,
        }
    }

    /*pub fn is_octothorpe(&self) -> bool {
        match self {
            Self::Octothorpe => true,
            _ => false,
        }
    }*/

    pub fn is_comment(&self) -> bool {
        match self {
            Self::Comment(_) => true,
            _ => false,
        }
    }

    pub fn is_equals(&self) -> bool {
        match self {
            Self::Equals => true,
            _ => false,
        }
    }

    pub fn is_let(&self) -> bool {
        match self {
            Self::Let => true,
            _ => false,
        }
    }

    pub fn is_open_angle_bracket(&self) -> bool {
        match self {
            Self::OpenAngleBracket => true,
            _ => false,
        }
    }

    pub fn is_close_angle_bracket(&self) -> bool {
        match self {
            Self::CloseAngleBracket => true,
            _ => false,
        }
    }

    pub fn is_dot(&self) -> bool {
        match self {
            Self::Dot => true,
            _ => false,
        }
    }

    pub fn is_exclamation(&self) -> bool {
        match self {
            Self::Exclamation => true,
            _ => false,
        }
    }

    pub fn is_interrogation(&self) -> bool {
        match self {
            Self::Interrogation => true,
            _ => false,
        }
    }
}

impl Display for MpsToken {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            //Self::Sql => write!(f, "sql"),
            Self::OpenBracket => write!(f, "("),
            Self::CloseBracket => write!(f, ")"),
            Self::Comma => write!(f, ","),
            Self::Literal(s) => write!(f, "\"{}\"", s),
            Self::Name(s) => write!(f, "{}", s),
            //Self::Octothorpe => write!(f, "#"),
            Self::Comment(s) => write!(f, "{}", s),
            Self::Equals => write!(f, "="),
            Self::Let => write!(f, "let"),
            Self::OpenAngleBracket => write!(f, "<"),
            Self::CloseAngleBracket => write!(f, ">"),
            Self::Dot => write!(f, "."),
            Self::Exclamation => write!(f, "!"),
            Self::Interrogation => write!(f, "?"),
        }
    }
}
