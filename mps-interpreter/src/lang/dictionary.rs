use std::collections::VecDeque;

use super::SyntaxError;
use super::{BoxedMpsOpFactory, MpsOp};
use crate::tokens::MpsToken;

pub struct MpsLanguageDictionary {
    vocabulary: Vec<Box<dyn BoxedMpsOpFactory>>,
}

impl MpsLanguageDictionary {
    pub fn add<T: BoxedMpsOpFactory + 'static>(&mut self, factory: T) -> &mut Self {
        self.vocabulary
            .push(Box::new(factory) as Box<dyn BoxedMpsOpFactory>);
        self
    }

    pub fn try_build_statement(
        &self,
        tokens: &mut VecDeque<MpsToken>,
    ) -> Result<Box<dyn MpsOp>, SyntaxError> {
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
                token: MpsToken::Name("???".into()),
                got: None,
            })
        }?;
        Err(SyntaxError {
            line: 0,
            token: MpsToken::Name("???".into()),
            got: Some(result),
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
