use std::iter::Iterator;
use std::fmt::{Debug, Display, Formatter, Error};

use std::collections::VecDeque;

use crate::tokens::MpsToken;
use crate::MpsMusicItem;

use super::SqlStatement;
use super::{SyntaxError, RuntimeError};
use super::MpsLanguageDictionary;

#[derive(Debug)]
pub enum MpsStatement {
    Sql(SqlStatement),
}

impl MpsStatement {
    pub fn eat_some(tokens: &mut VecDeque<MpsToken>, vocab: MpsLanguageDictionary) -> Result<Self, SyntaxError> {
        vocab.try_build_statement(tokens)
    }
}

impl Iterator for MpsStatement {
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MpsStatement::Sql(s) => s.next(),
        }
    }
}

impl Display for MpsStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Sql(s) => write!(f, "{}", s),
        }
    }
}
