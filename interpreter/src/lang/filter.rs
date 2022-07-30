use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;

use crate::lang::utility::{assert_name, assert_token, assert_token_raw, check_name};
use crate::lang::FilterReplaceStatement;
use crate::lang::LanguageDictionary;
use crate::lang::SingleItem;
use crate::lang::{BoxedOpFactory, IteratorItem, Op, PseudoOp};
use crate::lang::{RuntimeError, RuntimeMsg, RuntimeOp, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;
use crate::Item;

const INNER_VARIABLE_NAME: &str = "[inner variable]";

pub trait FilterPredicate: Clone + Debug + Display {
    fn matches(&mut self, item: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg>;

    fn is_complete(&self) -> bool;

    fn reset(&mut self) -> Result<(), RuntimeMsg>;
}

pub trait FilterFactory<P: FilterPredicate + 'static> {
    fn is_filter(&self, tokens: &VecDeque<&Token>) -> bool;

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<P, SyntaxError>;
}

#[derive(Debug, Clone)]
pub(super) enum VariableOrOp {
    Variable(String),
    Op(PseudoOp),
}

impl Display for VariableOrOp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Variable(s) => write!(f, "{}", s),
            Self::Op(op) => write!(f, "{}", op),
        }
    }
}

#[derive(Debug)]
pub struct FilterStatement<P: FilterPredicate + 'static> {
    predicate: P,
    iterable: VariableOrOp,
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
                "{}.({} || (like) {})",
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
            VariableOrOp::Variable(s) => {
                if self.context.is_some() {
                    let var = self.context.as_ref().unwrap().variables.get_opt(s);
                    if let Some(Type::Op(var)) = var {
                        var.is_resetable()
                    } else {
                        false
                    }
                } else {
                    true
                } // ASSUMPTION
            }
            VariableOrOp::Op(PseudoOp::Real(op)) => op.is_resetable(),
            VariableOrOp::Op(PseudoOp::Fake(_)) => false,
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
            VariableOrOp::Variable(s) => {
                if self.context.as_mut().unwrap().variables.exists(s) {
                    let mut var = self
                        .context
                        .as_mut()
                        .unwrap()
                        .variables
                        .remove(s)
                        .map_err(|e| e.with(RuntimeOp(fake.clone())))?;
                    let result = if let Type::Op(var) = &mut var {
                        var.enter(self.context.take().unwrap());
                        let result = var.reset();
                        self.context = Some(var.escape());
                        result
                    } else {
                        Err(RuntimeError {
                            line: 0,
                            op: fake.clone(),
                            msg: "Cannot reset non-iterable filter variable".to_string(),
                        })
                    };
                    self.context
                        .as_mut()
                        .unwrap()
                        .variables
                        .declare(s, var)
                        .map_err(|e| e.with(RuntimeOp(fake)))?;
                    result
                } else {
                    Ok(())
                }
            }
            VariableOrOp::Op(PseudoOp::Real(op)) => {
                op.enter(self.context.take().unwrap());
                let result = op.reset();
                self.context = Some(op.escape());
                result
            }
            VariableOrOp::Op(PseudoOp::Fake(_)) => Err(RuntimeError {
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
            iterable: match &self.iterable {
                VariableOrOp::Variable(s) => VariableOrOp::Variable(s.clone()),
                VariableOrOp::Op(op) => VariableOrOp::Op(op.try_real_ref().unwrap().dup().into()),
            },
            context: None,
            other_filters: self
                .other_filters
                .as_ref()
                .map(|x| PseudoOp::from(x.try_real_ref().unwrap().dup())),
            is_failing: false,
        })
    }
}

impl<P: FilterPredicate + 'static> Iterator for FilterStatement<P> {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.predicate.is_complete() && self.other_filters.is_none()) || self.is_failing {
            return None;
        }
        //let self_clone = self.clone();
        //let self_clone2 = self_clone.clone();
        let fake = PseudoOp::Fake(format!("{}", self));
        //let ctx = self.context.as_mut().unwrap();
        match &mut self.iterable {
            VariableOrOp::Op(op) => match op.try_real() {
                Ok(real_op) => {
                    let ctx = self.context.take().unwrap();
                    real_op.enter(ctx);
                    let mut maybe_result = None;
                    while let Some(item) = real_op.next() {
                        let mut ctx = real_op.escape();
                        match item {
                            Err(e) => {
                                //self.context = Some(op.escape());
                                maybe_result = Some(Err(e));
                                self.context = Some(ctx);
                                break;
                            }
                            Ok(item) => {
                                let matches_result = self.predicate.matches(&item, &mut ctx);
                                let matches = match matches_result {
                                    Err(e) => {
                                        maybe_result = Some(Err(e.with(RuntimeOp(fake))));
                                        self.context = Some(ctx);
                                        break;
                                    }
                                    Ok(b) => b,
                                };
                                if let Some(inner) = &mut self.other_filters {
                                    // handle other filters
                                    // make fake inner item
                                    let single_op = SingleItem::new_ok(item.clone());
                                    match ctx.variables.declare(
                                        INNER_VARIABLE_NAME,
                                        Type::Op(Box::new(single_op)),
                                    ) {
                                        Ok(x) => x,
                                        Err(e) => {
                                            //self.context = Some(op.escape());
                                            maybe_result =
                                                Some(Err(e.with(RuntimeOp(fake))));
                                            self.context = Some(ctx);
                                            break;
                                        }
                                    };
                                    let inner_real = match inner.try_real() {
                                        Ok(x) => x,
                                        Err(e) => {
                                            //self.context = Some(op.escape());
                                            maybe_result = Some(Err(e));
                                            self.context = Some(ctx);
                                            break;
                                        }
                                    };
                                    inner_real.enter(ctx);
                                    match inner_real.next() {
                                        Some(item) => {
                                            maybe_result = Some(item);
                                            ctx = inner_real.escape();
                                            match ctx.variables.remove(INNER_VARIABLE_NAME) {
                                                Ok(_) => {}
                                                Err(e) => match maybe_result {
                                                    Some(Ok(_)) => {
                                                        maybe_result = Some(Err(
                                                            e.with(RuntimeOp(fake))
                                                        ))
                                                    }
                                                    Some(Err(e2)) => maybe_result = Some(Err(e2)), // already failing, do not replace error,
                                                    None => {} // impossible
                                                },
                                            }
                                            self.context = Some(ctx);
                                            break;
                                        }
                                        None => {
                                            ctx = inner_real.escape(); // move ctx back to expected spot
                                            match ctx.variables.remove(INNER_VARIABLE_NAME) {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    //self.context = Some(op.escape());
                                                    maybe_result =
                                                        Some(Err(e.with(RuntimeOp(fake))));
                                                    self.context = Some(ctx);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                if matches {
                                    //self.context = Some(op.escape());
                                    maybe_result = Some(Ok(item));
                                    self.context = Some(ctx);
                                    break;
                                }
                            }
                        }
                        real_op.enter(ctx);
                    }
                    if self.context.is_none() {
                        self.context = Some(real_op.escape());
                    }
                    maybe_result
                }
                Err(e) => Some(Err(e)),
            },
            VariableOrOp::Variable(variable_name) => {
                let mut variable = match self
                    .context
                    .as_mut()
                    .unwrap()
                    .variables
                    .remove(variable_name)
                {
                    Ok(Type::Op(op)) => op,
                    Ok(x) => {
                        return Some(Err(RuntimeError {
                            line: 0,
                            op: fake,
                            msg: format!(
                                "Expected operation/iterable type in variable {}, got {}",
                                &variable_name, x
                            ),
                        }))
                    }
                    Err(e) => {
                        self.is_failing = true; // this is unrecoverable and reproducible, so it shouldn't be tried again (to prevent error spam)
                        return Some(Err(e.with(RuntimeOp(fake))))
                    },
                };
                let mut maybe_result = None;
                let ctx = self.context.take().unwrap();
                variable.enter(ctx);
                while let Some(item) = variable.next() {
                    let mut ctx = variable.escape();
                    match item {
                        Err(e) => {
                            maybe_result = Some(Err(e));
                            self.context = Some(ctx);
                            break;
                        }
                        Ok(item) => {
                            let matches_result = self.predicate.matches(&item, &mut ctx);
                            let matches = match matches_result {
                                Err(e) => {
                                    maybe_result = Some(Err(e.with(RuntimeOp(fake.clone()))));
                                    self.context = Some(ctx);
                                    break;
                                }
                                Ok(b) => b,
                            };
                            if let Some(inner) = &mut self.other_filters {
                                // handle other filters
                                // make fake inner item
                                let single_op = SingleItem::new_ok(item.clone());
                                match ctx
                                    .variables
                                    .declare(INNER_VARIABLE_NAME, Type::Op(Box::new(single_op)))
                                {
                                    Ok(x) => x,
                                    Err(e) => {
                                        //self.context = Some(op.escape());
                                        maybe_result = Some(Err(e.with(RuntimeOp(fake.clone()))));
                                        self.context = Some(ctx);
                                        break;
                                    }
                                };
                                let inner_real = match inner.try_real() {
                                    Ok(x) => x,
                                    Err(e) => {
                                        //self.context = Some(op.escape());
                                        maybe_result = Some(Err(e));
                                        self.context = Some(ctx);
                                        break;
                                    }
                                };
                                inner_real.enter(ctx);
                                match inner_real.next() {
                                    Some(item) => {
                                        maybe_result = Some(item);
                                        ctx = inner_real.escape();
                                        match ctx.variables.remove(INNER_VARIABLE_NAME) {
                                            Ok(_) => {}
                                            Err(e) => match maybe_result {
                                                Some(Ok(_)) => {
                                                    maybe_result =
                                                        Some(Err(e.with(RuntimeOp(fake.clone()))))
                                                }
                                                Some(Err(e2)) => maybe_result = Some(Err(e2)), // already failing, do not replace error,
                                                None => {} // impossible
                                            },
                                        }
                                        self.context = Some(ctx);
                                        break;
                                    }
                                    None => {
                                        ctx = inner_real.escape(); // move ctx back to expected spot
                                        match ctx.variables.remove(INNER_VARIABLE_NAME) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                //self.context = Some(op.escape());
                                                maybe_result =
                                                    Some(Err(e.with(RuntimeOp(fake.clone()))));
                                                self.context = Some(ctx);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            if matches {
                                maybe_result = Some(Ok(item));
                                self.context = Some(ctx);
                                break;
                            }
                        }
                    }
                    variable.enter(ctx);
                }
                if self.context.is_none() {
                    self.context = Some(variable.escape());
                }
                match self
                    .context
                    .as_mut()
                    .unwrap()
                    .variables
                    .declare(variable_name, Type::Op(variable))
                {
                    Err(e) => Some(Err(e.with(RuntimeOp(fake)))),
                    Ok(_) => maybe_result,
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.iterable {
            VariableOrOp::Variable(s) => self
                .context
                .as_ref()
                .and_then(|x| x.variables.get_opt(s))
                .and_then(|x| match x {
                    Type::Op(op) => Some(op.size_hint()),
                    _ => None,
                }),
            VariableOrOp::Op(op) => op.try_real_ref().map(|x| x.size_hint()).ok(),
        }
        .unwrap_or((0, None))
    }
}

pub struct FilterStatementFactory<
    P: FilterPredicate + 'static,
    F: FilterFactory<P> + 'static,
> {
    filter_factory: F,
    idc: PhantomData<P>,
}

impl<P: FilterPredicate + 'static, F: FilterFactory<P> + 'static>
    FilterStatementFactory<P, F>
{
    pub fn new(factory: F) -> Self {
        Self {
            filter_factory: factory,
            idc: PhantomData::<P>,
        }
    }
}

impl<P: FilterPredicate + 'static, F: FilterFactory<P> + 'static> BoxedOpFactory
    for FilterStatementFactory<P, F>
{
    #[allow(clippy::unnecessary_unwrap)]
    fn is_op_boxed(&self, tokens: &VecDeque<Token>) -> bool {
        let tokens_len = tokens.len();
        if is_correct_format(tokens) {
            let start_of_predicate = last_dot_before_open_bracket(tokens) + 2; // .(predicate)
            if start_of_predicate > tokens_len - 1 {
                false
            } else {
                let pipe_location_opt = first_double_pipe(tokens, 1);
                if pipe_location_opt.is_some() && pipe_location_opt.unwrap() > start_of_predicate {
                    let pipe_location = pipe_location_opt.unwrap();
                    // filters combined by OR operations
                    let tokens2: VecDeque<&Token> =
                        VecDeque::from_iter(tokens.range(start_of_predicate..pipe_location));
                    self.filter_factory.is_filter(&tokens2)
                } else {
                    // single filter
                    let tokens2: VecDeque<&Token> =
                        VecDeque::from_iter(tokens.range(start_of_predicate..tokens_len - 1));
                    if !tokens2.is_empty() && check_name("if", tokens2[0]) {
                        // replacement filter
                        if let Some(colon_location) = first_colon2(&tokens2) {
                            let tokens3 = VecDeque::from_iter(tokens.range(
                                start_of_predicate + 1..start_of_predicate + colon_location,
                            ));
                            self.filter_factory.is_filter(&tokens3)
                        } else {
                            false
                        }
                    } else {
                        // regular filter
                        self.filter_factory.is_filter(&tokens2)
                    }
                }
            }
        } else {
            false
        }
    }

    fn build_op_boxed(
        &self,
        tokens: &mut VecDeque<Token>,
        dict: &LanguageDictionary,
    ) -> Result<Box<dyn Op>, SyntaxError> {
        let start_of_op = last_dot_before_open_bracket(tokens);
        let op = if start_of_op == 1 && tokens[0].is_name() {
            // variable_name.(predicate)
            let variable_name = assert_token(
                |t| match t {
                    Token::Name(s) => Some(s),
                    _ => None,
                },
                Token::Name("variable_name".into()),
                tokens,
            )?;
            VariableOrOp::Variable(variable_name)
        } else {
            // <some other op>.(predicate)
            //let mut new_tokens = tokens.range(0..start_of_op).map(|x| x.to_owned()).collect();
            let end_tokens = tokens.split_off(start_of_op); // don't parse filter in inner statement
            let inner_op = dict.try_build_statement(tokens)?;
            tokens.extend(end_tokens);
            VariableOrOp::Op(inner_op.into())
        };
        assert_token_raw(Token::Dot, tokens)?;
        assert_token_raw(Token::OpenBracket, tokens)?;
        if !tokens.is_empty() && check_name("if", &tokens[0]) {
            return {
                // replacement filter
                //println!("Building replacement filter from tokens {:?}", tokens);
                assert_name("if", tokens)?;
                if let Some(colon_location) = first_colon(tokens) {
                    let end_tokens = tokens.split_off(colon_location);
                    let filter = self.filter_factory.build_filter(tokens, dict)?;
                    tokens.extend(end_tokens);
                    assert_token_raw(Token::Colon, tokens)?;
                    let mut else_op: Option<PseudoOp> = None;
                    let if_op: PseudoOp;
                    if let Some(else_location) = first_else_not_in_bracket(tokens) {
                        let end_tokens = tokens.split_off(else_location);
                        // build replacement system
                        if_op = dict.try_build_statement(tokens)?.into();
                        tokens.extend(end_tokens);
                        assert_name("else", tokens)?;
                        let end_tokens = tokens.split_off(tokens.len() - 1); // up to ending close bracket
                                                                             // build replacement system
                        else_op = Some(dict.try_build_statement(tokens)?.into());
                        tokens.extend(end_tokens);
                    } else {
                        let end_tokens = tokens.split_off(tokens.len() - 1);
                        // build replacement system
                        if_op = dict.try_build_statement(tokens)?.into();
                        tokens.extend(end_tokens);
                    }
                    assert_token_raw(Token::CloseBracket, tokens)?;
                    Ok(Box::new(FilterReplaceStatement {
                        predicate: filter,
                        iterable: op,
                        context: None,
                        op_if: if_op,
                        op_else: else_op,
                        item_cache: super::filter_replace::item_cache_deque(),
                    }))
                } else {
                    Err(SyntaxError {
                        line: 0,
                        token: Token::Colon,
                        got: None,
                    })
                }
            };
        } else {
            let mut another_filter = None;
            let (has_or, end_tokens) = if let Some(pipe_location) = first_double_pipe(tokens, 0) {
                (true, tokens.split_off(pipe_location)) // parse up to OR operator
            } else {
                (false, tokens.split_off(tokens.len() - 1)) // don't parse closing bracket in filter
            };
            let filter = self.filter_factory.build_filter(tokens, dict)?;
            tokens.extend(end_tokens);
            if has_or {
                // recursively build other filters for OR operation
                assert_token_raw(Token::Pipe, tokens)?;
                assert_token_raw(Token::Pipe, tokens)?;
                // emit fake filter syntax
                tokens.push_front(Token::OpenBracket);
                tokens.push_front(Token::Dot);
                tokens.push_front(Token::Name(INNER_VARIABLE_NAME.into())); // impossible to obtain through parsing on purpose
                another_filter = Some(dict.try_build_statement(tokens)?.into());
            } else {
                assert_token_raw(Token::CloseBracket, tokens)?; // remove closing bracket
            }
            Ok(Box::new(FilterStatement {
                predicate: filter,
                iterable: op,
                context: None,
                other_filters: another_filter,
                is_failing: false,
            }))
        }
    }
}

fn is_correct_format(tokens: &VecDeque<Token>) -> bool {
    let mut inside_brackets = 0;
    let mut open_bracket_found = false;
    let mut close_bracket = 0;
    for i in (0..tokens.len()).rev() {
        if tokens[i].is_close_bracket() {
            if inside_brackets == 0 {
                close_bracket = i;
            }
            inside_brackets += 1;
        } else if tokens[i].is_open_bracket() {
            if inside_brackets == 1 {
                open_bracket_found = true;
            } else if inside_brackets != 0 {
                inside_brackets -= 1;
            }
        } else if open_bracket_found {
            return tokens[i].is_dot() && (close_bracket == 0 || close_bracket == tokens.len() - 1);
        }
    }
    false
}

fn last_dot_before_open_bracket(tokens: &VecDeque<Token>) -> usize {
    let mut inside_brackets = 0;
    let mut open_bracket_found = false;
    for i in (0..tokens.len()).rev() {
        if tokens[i].is_close_bracket() {
            inside_brackets += 1;
        } else if tokens[i].is_open_bracket() {
            if inside_brackets == 1 {
                open_bracket_found = true;
            } else if inside_brackets != 0 {
                inside_brackets -= 1;
            }
        } else if open_bracket_found {
            if tokens[i].is_dot() {
                return i;
            } else {
                return 0;
            }
        }
    }
    0
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

fn first_colon(tokens: &VecDeque<Token>) -> Option<usize> {
    for i in 0..tokens.len() {
        if tokens[i].is_colon() {
            return Some(i);
        }
    }
    None
}

fn first_colon2(tokens: &VecDeque<&Token>) -> Option<usize> {
    for i in 0..tokens.len() {
        if tokens[i].is_colon() {
            return Some(i);
        }
    }
    None
}

fn first_else_not_in_bracket(tokens: &VecDeque<Token>) -> Option<usize> {
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
}
