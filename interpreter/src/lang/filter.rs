use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;

use crate::lang::utility::{assert_name, assert_token_raw, check_name};
use crate::lang::FilterReplaceStatement;
use crate::lang::LanguageDictionary;
use crate::lang::SingleItem;
use crate::lang::{BoxedTransformOpFactory, IteratorItem, Op, PseudoOp};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;
use crate::Item;

const INNER_VARIABLE_NAME: &str = "[inner variable]";

pub trait FilterPredicate: Clone + Debug + Display + Send {
    fn matches(&mut self, item: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg>;

    fn is_complete(&self) -> bool;

    fn reset(&mut self) -> Result<(), RuntimeMsg>;
}

pub trait FilterFactory<P: FilterPredicate + 'static>: Send + Sync {
    fn is_filter(&self, tokens: &VecDeque<&Token>) -> bool;

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<P, SyntaxError>;
}

#[derive(Debug)]
pub struct FilterStatement<P: FilterPredicate + 'static> {
    predicate: P,
    iterable: PseudoOp,
    context: Option<Context>,
    other_filters: Option<PseudoOp>,
    is_failing: bool,
}

impl<P: FilterPredicate + 'static> std::clone::Clone for FilterStatement<P> {
    fn clone(&self) -> Self {
        Self {
            predicate: self.predicate.clone(),
            iterable: self.iterable.clone(),
            context: None,
            other_filters: self.other_filters.clone(),
            is_failing: self.is_failing,
        }
    }
}

impl<P: FilterPredicate + 'static> Display for FilterStatement<P> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(other_filters) = &self.other_filters {
            write!(
                f,
                "{}.({} || (retconned) {})",
                self.iterable, self.predicate, other_filters
            )
        } else {
            write!(f, "{}.({})", self.iterable, self.predicate)
        }
    }
}

impl<P: FilterPredicate + 'static> Op for FilterStatement<P> {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        let is_iterable_resetable = match &self.iterable {
            PseudoOp::Real(op) => op.is_resetable(),
            PseudoOp::Fake(_) => false,
        };
        let is_other_filter_resetable =
            if let Some(PseudoOp::Real(other_filter)) = &self.other_filters {
                other_filter.is_resetable()
            } else {
                true
            };
        is_iterable_resetable && is_other_filter_resetable
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        let fake = PseudoOp::Fake(format!("{}", self));
        self.is_failing = false;
        self.predicate
            .reset()
            .map_err(|x| x.with(RuntimeOp(fake.clone())))?;
        match &mut self.iterable {
            PseudoOp::Real(op) => {
                op.enter(self.context.take().unwrap());
                let result = op.reset();
                self.context = Some(op.escape());
                result
            }
            PseudoOp::Fake(_) => Err(RuntimeError {
                line: 0,
                op: fake,
                msg: "Cannot reset PseudoOp::Fake filter".to_string(),
            }),
        }?;
        if let Some(PseudoOp::Real(other_filter)) = &mut self.other_filters {
            other_filter.enter(self.context.take().unwrap());
            let result = other_filter.reset();
            self.context = Some(other_filter.escape());
            result
        } else {
            Ok(())
        }
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(Self {
            predicate: self.predicate.clone(),
            iterable: self.iterable.try_real_ref().unwrap().dup().into(),
            context: None,
            other_filters: self
                .other_filters
                .as_ref()
                .map(|x| PseudoOp::from(x.try_real_ref().unwrap().dup())),
            is_failing: false,
        })
    }
}

impl <P: FilterPredicate + 'static> FilterStatement<P> {
    fn next_item(&mut self) -> Option<IteratorItem> {
        match self.iterable.try_real() {
            Ok(real_op) => {
                let ctx = self.context.take().unwrap();
                real_op.enter(ctx);
                let item = real_op.next();
                self.context = Some(real_op.escape());
                item
            }
            Err(e) => return Some(Err(e)),
        }
    }
}

impl<P: FilterPredicate + 'static> Iterator for FilterStatement<P> {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        //println!("In FilterStatement {}", self.predicate);
        if (self.predicate.is_complete() && self.other_filters.is_none()) || self.is_failing {
            return None;
        }
        //let ctx = self.context.as_mut().unwrap();
        while let Some(next_item) = self.next_item() {
            match next_item {
                Ok(item) => {
                    //let ctx = self.context.as_mut().unwrap();
                    let matches_result = self.predicate.matches(&item, self.context.as_mut().unwrap());
                    let matches = match matches_result {
                        Err(e) => {
                            //maybe_result = Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))));
                            //self.context = Some(ctx);
                            return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))));
                        }
                        Ok(b) => b,
                    };
                    if let Some(inner) = &mut self.other_filters {
                        // handle other filters
                        // make fake inner item
                        //println!("Making fake inner variable `{}`", INNER_VARIABLE_NAME);
                        let single_op = SingleItem::new_ok(item.clone());
                        let preexisting_var = self.context.as_mut().unwrap()
                            .variables
                            .swap(INNER_VARIABLE_NAME, Some(Type::Op(Box::new(single_op))));
                        let inner_real = match inner.try_real() {
                            Ok(x) => x,
                            Err(e) => return Some(Err(e))
                        };
                        inner_real.enter(self.context.take().unwrap());
                        match inner_real.next() {
                            Some(item) => {
                                self.context = Some(inner_real.escape());
                                self.context.as_mut().unwrap().variables.swap(INNER_VARIABLE_NAME, preexisting_var);
                                return Some(item);
                            }
                            None => {
                                self.context = Some(inner_real.escape()); // move ctx back to expected spot
                                self.context.as_mut().unwrap().variables.swap(INNER_VARIABLE_NAME, preexisting_var);
                            }
                        }
                    }
                    if matches {
                        return Some(Ok(item));
                    }
                },
                Err(e) => return Some(Err(e)),
            };
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterable.try_real_ref().map(|x| x.size_hint()).ok().unwrap_or((0, None))
    }
}

pub struct FilterStatementFactory<P: FilterPredicate + 'static, F: FilterFactory<P> + 'static> {
    filter_factory: F,
    idc: PhantomData<P>,
}

impl<P: FilterPredicate + 'static, F: FilterFactory<P> + 'static> FilterStatementFactory<P, F> {
    pub fn new(factory: F) -> Self {
        Self {
            filter_factory: factory,
            idc: PhantomData::<P>,
        }
    }
}

impl<P: FilterPredicate + 'static, F: FilterFactory<P> + 'static> BoxedTransformOpFactory
    for FilterStatementFactory<P, F> {
    fn build_transform_op(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
        op: Box<dyn Op>,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        assert_token_raw(Token::Dot, tokens)?;
        assert_token_raw(Token::OpenBracket, tokens)?;
        if !tokens.is_empty() && check_name("if", &tokens[0]) {
            // replacement filter
            assert_name("if", tokens)?;
            let filter = self.filter_factory.build_filter(tokens, dict)?;
            assert_token_raw(Token::Colon, tokens)?;
            let mut else_op: Option<PseudoOp> = None;
            let if_op: PseudoOp = dict.try_build_statement(tokens)?.into();
            if check_name("else", &tokens[0]) {
                assert_name("else", tokens)?;
                else_op = Some(dict.try_build_statement(tokens)?.into());
            }
            /*if let Some(else_location) = first_else_not_in_bracket(tokens) {
                //println!("First else found at {}; {:?}", else_location, tokens);
                //let end_tokens = tokens.split_off(else_location);
                // build replacement system
                if_op = dict.try_build_statement(tokens)?.into();
                //tokens.extend(end_tokens);
                assert_name("else", tokens)?;
                //let end_tokens = tokens.split_off(tokens.len() - 1); // up to ending close bracket
                                                                        // build replacement system
                else_op = Some(dict.try_build_statement(tokens)?.into());
                //tokens.extend(end_tokens);
            } else {
                //let end_tokens = tokens.split_off(tokens.len() - 1);
                // build replacement system
                if_op = dict.try_build_statement(tokens)?.into();
                //tokens.extend(end_tokens);
            }*/
            assert_token_raw(Token::CloseBracket, tokens)?;
            Ok(Box::new(FilterReplaceStatement {
                predicate: filter,
                iterable:  op.into(),
                context: None,
                op_if: if_op,
                op_else: else_op,
                item_cache: super::filter_replace::item_cache_deque(),
            }))
        } else {
            // regular filter
            let mut another_filter = None;
            let has_or = first_double_pipe(tokens, 0).is_some();
            /*let (has_or, end_tokens) = if let Some(pipe_location) = first_double_pipe(tokens, 0) {
                (true, tokens.split_off(pipe_location)) // parse up to OR operator
            } else {
                (false, tokens.split_off(tokens.len() - 1)) // don't parse closing bracket in filter
            };*/
            let filter = self.filter_factory.build_filter(tokens, dict)?;
            //tokens.extend(end_tokens);
            if has_or {
                // recursively build other filters for OR operation
                assert_token_raw(Token::Pipe, tokens)?;
                assert_token_raw(Token::Pipe, tokens)?;
                // emit fake filter syntax
                tokens.push_front(Token::OpenBracket);
                tokens.push_front(Token::Dot);
                //tokens.push_front(Token::Name(INNER_VARIABLE_NAME.into())); // impossible to obtain through parsing on purpose
                /*let inner_op = Box::new(crate::lang::vocabulary::VariableRetrieveStatement {
                    variable_name: INNER_VARIABLE_NAME.into(),
                    context: None,
                    is_tried: false,
                });*/
                let mut inner_tokens = VecDeque::with_capacity(1);
                inner_tokens.push_front(Token::Name(INNER_VARIABLE_NAME.into()));
                let inner_op = dict.try_build_statement(&mut inner_tokens)?;
                let (inner_op, op_transformed) = dict.try_build_one_transform(inner_op, tokens)?;
                if !op_transformed {
                    return Err(
                        SyntaxError {
                            token: Token::Name(INNER_VARIABLE_NAME.into()),
                            got: Some(Token::Name(INNER_VARIABLE_NAME.into())),
                            line: 0,
                        }
                    )
                }
                //println!("Built 2nd filter: {}", inner_op);
                another_filter = Some(inner_op.into());
            } else {
                //println!("filter tokens before final bracket: {:?}", tokens);
                assert_token_raw(Token::CloseBracket, tokens)?; // remove closing bracket
            }
            Ok(Box::new(FilterStatement {
                predicate: filter,
                iterable:  op.into(),
                context: None,
                other_filters: another_filter,
                is_failing: false,
            }))
        }
    }

    fn is_transform_op(&self, tokens: &VecDeque<Token>) -> bool {
        let result = if tokens.len() > 2 && tokens[0].is_dot() && tokens[1].is_open_bracket() {
            if check_name("if", &tokens[2]) {
                let tokens2: VecDeque<&Token> =
                    VecDeque::from_iter(tokens.range(3..));
                self.filter_factory.is_filter(&tokens2)
            } else {
                let tokens2: VecDeque<&Token> =
                    VecDeque::from_iter(tokens.range(2..));
                self.filter_factory.is_filter(&tokens2)
            }
        } else {
            false
        };
        result
    }
}

fn first_double_pipe(tokens: &VecDeque<Token>, in_brackets: usize) -> Option<usize> {
    let mut inside_brackets = 0;
    let mut pipe_found = false;
    for i in 0..tokens.len() {
        if tokens[i].is_pipe() && inside_brackets == in_brackets {
            if pipe_found {
                return Some(i - 1);
            } else {
                pipe_found = true;
            }
        } else {
            pipe_found = false;
            if tokens[i].is_open_bracket() {
                inside_brackets += 1;
            } else if tokens[i].is_close_bracket() && inside_brackets != 0 {
                inside_brackets -= 1;
            }
        }
    }
    None
}

/*fn first_else_not_in_bracket(tokens: &VecDeque<Token>) -> Option<usize> {
    let mut inside_brackets = 0;
    for i in 0..tokens.len() {
        if check_name("else", &tokens[i]) && inside_brackets == 0 {
            return Some(i);
        } else if tokens[i].is_open_bracket() {
            inside_brackets += 1;
        } else if tokens[i].is_close_bracket() && inside_brackets != 0 {
            inside_brackets -= 1;
        }
    }
    None
}*/
