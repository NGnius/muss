use std::collections::VecDeque;
//use std::fmt::{Debug, Display, Error, Formatter};
use std::marker::PhantomData;

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::LanguageDictionary;
use crate::lang::SyntaxError;
use crate::lang::{BoxedOpFactory, Op};
use crate::tokens::Token;

pub trait FunctionFactory<O: Op + 'static>: Send + Sync {
    fn is_function(&self, name: &str) -> bool;

    fn build_function_params(
        &self,
        name: String,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<O, SyntaxError>;
}

pub struct FunctionStatementFactory<O: Op + 'static, F: FunctionFactory<O> + 'static> {
    op_factory: F,
    idc: PhantomData<O>,
}

impl<O: Op + 'static, F: FunctionFactory<O> + 'static> FunctionStatementFactory<O, F> {
    pub fn new(factory: F) -> Self {
        Self {
            op_factory: factory,
            idc: PhantomData::default(),
        }
    }
}

impl<O: Op + 'static, F: FunctionFactory<O> + 'static> BoxedOpFactory
    for FunctionStatementFactory<O, F>
{
    fn is_op_boxed(&self, tokens: &VecDeque<Token>) -> bool {
        let tokens_len = tokens.len();
        if tokens_len < 3 {
            false
        } else {
            match &tokens[0] {
                Token::Name(n) => {
                    self.op_factory.is_function(n)
                        && tokens[1].is_open_bracket()
                        && tokens[tokens_len - 1].is_close_bracket()
                }
                _ => false,
            }
        }
    }

    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        let name = assert_token(
            |t| match t {
                Token::Name(n) => Some(n),
                _ => None,
            },
            Token::Name("function_name".into()),
            tokens,
        )?;
        assert_token_raw(Token::OpenBracket, tokens)?;
        let end_tokens = tokens.split_off(tokens.len() - 1);
        let func = self.op_factory.build_function_params(name, tokens, dict)?;
        tokens.extend(end_tokens);
        assert_token_raw(Token::CloseBracket, tokens)?;
        Ok(Box::new(func))
    }
}
