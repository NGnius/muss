use std::collections::VecDeque;

use super::SyntaxError;
use super::{BoxedOpFactory, Op};
use crate::tokens::Token;

pub struct LanguageDictionary {
    vocabulary: Vec<Box<dyn BoxedOpFactory>>,
}

impl LanguageDictionary {
    pub fn add<T: BoxedOpFactory + 'static>(&mut self, factory: T) -> &mut Self {
        self.vocabulary
            .push(Box::new(factory) as Box<dyn BoxedOpFactory>);
        self
    }

    pub fn try_build_statement(
        &self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        //println!("try_build_statement with tokens {:?}", tokens);
        for factory in &self.vocabulary {
            if factory.is_op_boxed(tokens) {
                return factory.build_op_boxed(tokens, self);
            }
        }
        let result = match tokens.pop_front() {
            Some(x) => Ok(x),
            None => Err(SyntaxError {
                line: 0,
                token: Token::Name("{something}".into()),
                got: None,
            }),
        }?;
        Err(SyntaxError {
            line: 0,
            token: Token::Name("{any of many}".into()),
            got: Some(result),
        })
    }

    pub fn new() -> Self {
        Self {
            vocabulary: Vec::new(),
        }
    }

    pub fn standard() -> Self {
        let mut new = Self::new();
        crate::interpretor::standard_vocab(&mut new);
        new
    }
}

#[allow(clippy::derivable_impls)]
impl Default for LanguageDictionary {
    fn default() -> Self {
        Self {
            vocabulary: Vec::new(),
        }
    }
}
