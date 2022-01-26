use std::collections::VecDeque;
//use std::fmt::{Debug, Display, Error, Formatter};
use std::marker::PhantomData;

use crate::lang::utility::{assert_token, assert_token_raw, assert_token_raw_back};
#[cfg(debug_assertions)]
use crate::lang::utility::assert_empty;
use crate::lang::MpsLanguageDictionary;
use crate::lang::SyntaxError;
use crate::lang::{BoxedMpsOpFactory, MpsOp};
use crate::tokens::MpsToken;

pub trait MpsFunctionFactory<Op: MpsOp + 'static> {
    fn is_function(&self, name: &str) -> bool;

    fn build_function_params(
        &self,
        name: String,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Op, SyntaxError>;
}

pub struct MpsFunctionStatementFactory<Op: MpsOp + 'static, F: MpsFunctionFactory<Op> + 'static> {
    op_factory: F,
    idc: PhantomData<Op>,
}

impl<Op: MpsOp + 'static, F: MpsFunctionFactory<Op> + 'static> MpsFunctionStatementFactory<Op, F> {
    pub fn new(factory: F) -> Self {
        Self {
            op_factory: factory,
            idc: PhantomData::default(),
        }
    }
}

impl<Op: MpsOp + 'static, F: MpsFunctionFactory<Op> + 'static> BoxedMpsOpFactory
    for MpsFunctionStatementFactory<Op, F>
{
    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool {
        let tokens_len = tokens.len();
        if tokens_len < 3 {
            false
        } else {
            match &tokens[0] {
                MpsToken::Name(n) => {
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
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsOp>, SyntaxError> {
        let name = assert_token(
            |t| match t {
                MpsToken::Name(n) => Some(n),
                _ => None,
            },
            MpsToken::Name("function_name".into()),
            tokens,
        )?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        assert_token_raw_back(MpsToken::CloseBracket, tokens)?;
        let func = self.op_factory.build_function_params(name, tokens, dict)?;
        #[cfg(debug_assertions)]
        assert_empty(tokens)?;
        Ok(Box::new(func))
    }
}
