use std::collections::VecDeque;
use std::fmt::{Display, Error, Formatter};

//use super::MpsTypePrimitive;
use crate::processing::general::MpsType;
use crate::lang::{RuntimeError, SyntaxError};
use super::utility::{assert_type, assert_token, check_is_type};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::processing::OpGetter;

#[derive(Debug)]
pub enum Lookup {
    Static(MpsType),
    Variable(String)
}

impl Lookup {
    pub fn check_is(token: &MpsToken) -> bool {
        token.is_name() || check_is_type(token)
    }
    pub fn parse(tokens: &mut VecDeque<MpsToken>) -> Result<Self, SyntaxError> {
        if tokens.is_empty() {
            Err(SyntaxError {
                line: 0,
                token: MpsToken::Name("Float | UInt | Int | Bool".into()),
                got: None,
            })
        } else if check_is_type(&tokens[0]) {
            Ok(Self::Static(MpsType::Primitive(assert_type(tokens)?)))
        } else {
            Ok(Self::Variable(assert_token(|t| match t {
                MpsToken::Name(s) => Some(s),
                _ => None,
            }, MpsToken::Name("variable_name".into()), tokens)?))
        }
    }

    pub fn get_mut<'a, 'b: 'a>(&'b mut self, ctx: &'a mut MpsContext, op: &mut OpGetter) -> Result<&'a mut MpsType, RuntimeError> {
        match self {
            Self::Static(var) => Ok(var),
            Self::Variable(name) => ctx.variables.get_mut(name, op)
        }
    }

    pub fn get<'a, 'b: 'a>(&'b self, ctx: &'a MpsContext, op: &mut OpGetter) -> Result<&'a MpsType, RuntimeError> {
        match self {
            Self::Static(var) => Ok(var),
            Self::Variable(name) => ctx.variables.get(name, op)
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
                MpsType::Primitive(p) => Self::Static(MpsType::Primitive(p.clone())),
                _ => panic!("Can't clone static operator (invalid state)")
            },
            Self::Variable(name) => Self::Variable(name.clone()),
        }
    }
}
