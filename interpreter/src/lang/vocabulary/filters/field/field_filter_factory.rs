use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::{FilterFactory, FilterPredicate, FilterStatementFactory};
use crate::tokens::Token;
use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::LanguageDictionary;
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::Context;
use crate::Item;

pub trait FieldFilterFactory<T: FieldFilterPredicate + 'static>: Send + Sync {
    fn is_filter(&self, tokens: &[Token]) -> bool;

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        field: String,
        dict: &LanguageDictionary,
    ) -> Result<T, SyntaxError>;
}

pub trait FieldFilterPredicate: Send + Sync + Debug + Display {
    fn matches(&mut self, item: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg>;

    fn is_complete(&self) -> bool;

    fn reset(&mut self) -> Result<(), RuntimeMsg>;

    fn box_clone(&self) -> Box<dyn FieldFilterPredicate + 'static>;
}

pub struct FieldFilterFactoryBoxer<T: FieldFilterPredicate + 'static> {
    inner: Box<dyn FieldFilterFactory<T>>
}

#[derive(Debug)]
pub struct BoxedFilterPredicate {
    inner: Box<dyn FieldFilterPredicate + 'static>
}

impl Clone for BoxedFilterPredicate {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.box_clone()
        }
    }
}

impl Display for BoxedFilterPredicate {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        (&self.inner as &dyn Display).fmt(f)
    }
}

impl FilterPredicate for BoxedFilterPredicate {
    fn matches(&mut self, item: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg> {
        self.inner.matches(item, ctx)
    }

    fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        self.inner.reset()
    }
}

impl FieldFilterPredicate for BoxedFilterPredicate {
    fn matches(&mut self, item: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg> {
        self.inner.matches(item, ctx)
    }

    fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        self.inner.reset()
    }

    fn box_clone(&self) -> Box<dyn FieldFilterPredicate + 'static> {
        self.inner.box_clone()
    }
}

impl <T: FieldFilterPredicate + 'static> FieldFilterFactory<BoxedFilterPredicate> for FieldFilterFactoryBoxer<T> {
    fn is_filter(&self, tokens: &[Token]) -> bool {
        self.inner.is_filter(tokens)
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        field: String,
        dict: &LanguageDictionary,
    ) -> Result<BoxedFilterPredicate, SyntaxError> {
        self.inner.build_filter(tokens, field, dict).map(|x| BoxedFilterPredicate { inner: Box::new(x) })
    }
}

pub struct FieldFilterBlockFactory {
    field_filters: Vec<Box<dyn FieldFilterFactory<BoxedFilterPredicate>>>,
}

impl FieldFilterBlockFactory {
    pub fn new() -> Self {
        Self {
            field_filters: Vec::new(),
        }
    }

    pub fn push<T: FieldFilterPredicate + 'static, F: FieldFilterFactory<T> + 'static>(mut self, factory: F) -> Self {
        self.field_filters.push(
            Box::new(FieldFilterFactoryBoxer { inner: Box::new(factory) })
        );
        self
    }

    #[inline(always)]
    pub fn to_statement_factory(self) -> FieldFilterStatementFactory {
        FieldFilterStatementFactory::new(self)
    }
}

impl FilterFactory<BoxedFilterPredicate> for FieldFilterBlockFactory {
    fn is_filter(&self, tokens: &VecDeque<&Token>) -> bool {
        tokens.len() > 1 && tokens[0].is_dot()
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<BoxedFilterPredicate, SyntaxError> {
        assert_token_raw(Token::Dot, tokens)?;
        let field = assert_token(
            |t| match t {
                Token::Name(n) => Some(n),
                _ => None,
            },
            Token::Name("field_name".into()),
            tokens,
        )?;
        for filter in &self.field_filters {
            if filter.is_filter(tokens.make_contiguous()) {
                return filter.build_filter(tokens, field, dict);
            }
        }
        Err(SyntaxError {
            got: tokens.front().cloned(),
            token: Token::Name("<comparison op>".into()),
            line: 0,
        })
    }
}

pub type FieldFilterStatementFactory =
    FilterStatementFactory<BoxedFilterPredicate, FieldFilterBlockFactory>;
