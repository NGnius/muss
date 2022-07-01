use std::collections::VecDeque;
use std::fmt::{Display, Error, Formatter};

//use super::TypePrimitive;
use super::utility::{assert_token, assert_type, check_is_type};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;

#[derive(Debug)]
pub enum Lookup {
    Static(Type),
    Variable(String),
}

impl Lookup {
    pub fn check_is(token: &Token) -> bool {
        token.is_name() || check_is_type(token)
    }

    pub fn parse(tokens: &mut VecDeque<Token>) -> Result<Self, SyntaxError> {
        if tokens.is_empty() {
            Err(SyntaxError {
                line: 0,
                token: Token::Name("Float | UInt | Int | Bool".into()),
                got: None,
            })
        } else if check_is_type(&tokens[0]) {
            Ok(Self::Static(Type::Primitive(assert_type(tokens)?)))
        } else {
            Ok(Self::Variable(assert_token(
                |t| match t {
                    Token::Name(s) => Some(s),
                    _ => None,
                },
                Token::Name("variable_name".into()),
                tokens,
            )?))
        }
    }

    pub fn get_mut<'a, 'b: 'a>(
        &'b mut self,
        ctx: &'a mut Context,
    ) -> Result<&'a mut Type, RuntimeMsg> {
        match self {
            Self::Static(var) => Ok(var),
            Self::Variable(name) => ctx.variables.get_mut(name),
        }
    }

    pub fn get<'a, 'b: 'a>(&'b self, ctx: &'a Context) -> Result<&'a Type, RuntimeMsg> {
        match self {
            Self::Static(var) => Ok(var),
            Self::Variable(name) => ctx.variables.get(name),
        }
    }
}

impl Display for Lookup {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Static(var) => write!(f, "{}", var),
            Self::Variable(name) => write!(f, "{}", name),
        }
    }
}

impl std::clone::Clone for Lookup {
    fn clone(&self) -> Self {
        match self {
            Self::Static(var) => match var {
                Type::Primitive(p) => Self::Static(Type::Primitive(p.clone())),
                _ => panic!("Can't clone static operator (invalid state)"),
            },
            Self::Variable(name) => Self::Variable(name.clone()),
        }
    }
}
