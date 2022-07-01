use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
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
    Pipe,
    Ampersand,
    Colon,
    Tilde,
    OpenCurly,
    CloseCurly,
    Plus,
    Minus,
}

impl Token {
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
            "|" => Ok(Self::Pipe),
            "&" => Ok(Self::Ampersand),
            ":" => Ok(Self::Colon),
            "~" => Ok(Self::Tilde),
            "{" => Ok(Self::OpenCurly),
            "}" => Ok(Self::CloseCurly),
            "+" => Ok(Self::Plus),
            "-" => Ok(Self::Minus),
            _ => {
                // name validation
                let mut ok = true;
                for invalid_c in [
                    "-", "+", ",", " ", "/", "\n", "\r", "!", "?", "=", ".", "&", "|",
                ] {
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

    pub fn is_pipe(&self) -> bool {
        match self {
            Self::Pipe => true,
            _ => false,
        }
    }

    pub fn is_ampersand(&self) -> bool {
        match self {
            Self::Ampersand => true,
            _ => false,
        }
    }

    pub fn is_colon(&self) -> bool {
        match self {
            Self::Colon => true,
            _ => false,
        }
    }

    pub fn is_tilde(&self) -> bool {
        match self {
            Self::Tilde => true,
            _ => false,
        }
    }

    pub fn is_open_curly(&self) -> bool {
        match self {
            Self::OpenCurly => true,
            _ => false,
        }
    }

    pub fn is_close_curly(&self) -> bool {
        match self {
            Self::CloseCurly => true,
            _ => false,
        }
    }

    pub fn is_plus(&self) -> bool {
        match self {
            Self::Plus => true,
            _ => false,
        }
    }

    pub fn is_minus(&self) -> bool {
        match self {
            Self::Minus => true,
            _ => false,
        }
    }
}

impl Display for Token {
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
            Self::Pipe => write!(f, "|"),
            Self::Ampersand => write!(f, "&"),
            Self::Colon => write!(f, ":"),
            Self::Tilde => write!(f, "~"),
            Self::OpenCurly => write!(f, "{{"),
            Self::CloseCurly => write!(f, "}}"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
        }
    }
}
