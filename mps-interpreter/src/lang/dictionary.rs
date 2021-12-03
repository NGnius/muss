use std::collections::VecDeque;

use crate::tokens::MpsToken;
use super::{BoxedMpsOpFactory, MpsOp};
use super::SyntaxError;

pub struct MpsLanguageDictionary {
    vocabulary: Vec<Box<dyn BoxedMpsOpFactory>>
}

impl MpsLanguageDictionary {
    pub fn add<T: BoxedMpsOpFactory + 'static>(&mut self, factory: T) -> &mut Self {
        self.vocabulary.push(Box::new(factory) as Box<dyn BoxedMpsOpFactory>);
        self
    }

    pub fn try_build_statement(&self, tokens: &mut VecDeque<MpsToken>) -> Result<Box<dyn MpsOp>, SyntaxError> {
        for factory in &self.vocabulary {
            if factory.is_op_boxed(tokens) {
                return factory.build_op_boxed(tokens);
            }
        }
        Err(SyntaxError {
            line: 0,
            token: tokens.pop_front().unwrap()
        })
    }

    pub fn new() -> Self {
        Self {
            vocabulary: Vec::new(),
        }
    }
}

impl Default for MpsLanguageDictionary {
    fn default() -> Self {
        Self {
            vocabulary: Vec::new(),
        }
    }
}
