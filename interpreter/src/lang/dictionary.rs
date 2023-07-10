use std::collections::VecDeque;

use super::SyntaxError;
use super::{BoxedOpFactory, Op, BoxedTransformOpFactory};
use crate::tokens::Token;

pub struct LanguageDictionary {
    root_vocabulary: Vec<Box<dyn BoxedOpFactory>>,
    transform_vocabulary: Vec<Box<dyn BoxedTransformOpFactory>>,
}

impl LanguageDictionary {
    pub fn add<T: BoxedOpFactory + 'static>(&mut self, factory: T) -> &mut Self {
        self.root_vocabulary
            .push(Box::new(factory) as Box<dyn BoxedOpFactory>);
        self
    }

    pub fn add_transform<T: BoxedTransformOpFactory + 'static>(&mut self, factory: T) -> &mut Self {
        self.transform_vocabulary
            .push(Box::new(factory) as Box<dyn BoxedTransformOpFactory>);
        self
    }

    fn try_build_root_statement(
        &self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        //println!("building root op with tokens {:?}", tokens);
        for factory in &self.root_vocabulary {
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

    fn try_build_transformed_statement(
        &self,
        mut op: Box<dyn Op>,
        tokens: &mut VecDeque<Token>,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        //println!("building transformer for op {} with tokens {:?}", op, tokens);
        let mut op_found = true;
        while op_found && !tokens.is_empty() {
            (op, op_found) = self.try_build_one_transform(op, tokens)?;
        }
        //println!("built transformed op {}, remaining tokens {:?}", op, tokens);
        Ok(op)
    }

    pub fn try_build_one_transform(
        &self,
        mut op: Box<dyn Op>,
        tokens: &mut VecDeque<Token>,
    ) -> Result<(Box<dyn Op>, bool), SyntaxError> {
        for factory in &self.transform_vocabulary {
            if factory.is_transform_op(tokens) {
                op = factory.build_transform_op(tokens, self, op)?;
                return Ok((op, true))
            }
        }
        Ok((op, false))
    }

    pub fn try_build_statement(
        &self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        let root = self.try_build_root_statement(tokens)?;
        //println!("built root op {}, remaining tokens {:?}", root, tokens);
        self.try_build_transformed_statement(root, tokens)
    }

    pub fn new() -> Self {
        Self {
            root_vocabulary: Vec::new(),
            transform_vocabulary: Vec::new(),
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
            root_vocabulary: Vec::new(),
            transform_vocabulary: Vec::new(),
        }
    }
}
