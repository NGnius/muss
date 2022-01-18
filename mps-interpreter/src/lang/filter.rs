use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;

use crate::lang::utility::{assert_token, assert_token_raw, check_name, assert_name};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{BoxedMpsOpFactory, MpsOp, PseudoOp};
use crate::lang::{RuntimeError, SyntaxError};
use crate::lang::SingleItem;
use crate::lang::MpsFilterReplaceStatement;
use crate::processing::general::MpsType;
use crate::processing::OpGetter;
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsMusicItem;

const INNER_VARIABLE_NAME: &str = "[inner variable]";

pub trait MpsFilterPredicate: Clone + Debug + Display {
    fn matches(
        &mut self,
        item: &MpsMusicItem,
        ctx: &mut MpsContext,
        op: &mut OpGetter,
    ) -> Result<bool, RuntimeError>;

    fn is_complete(&self) -> bool;

    fn reset(&mut self) -> Result<(), RuntimeError>;
}

pub trait MpsFilterFactory<P: MpsFilterPredicate + 'static> {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool;

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
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
pub struct MpsFilterStatement<P: MpsFilterPredicate + 'static> {
    predicate: P,
    iterable: VariableOrOp,
    context: Option<MpsContext>,
    other_filters: Option<PseudoOp>,
}

impl<P: MpsFilterPredicate + 'static> std::clone::Clone for MpsFilterStatement<P> {
    fn clone(&self) -> Self {
        Self {
            predicate: self.predicate.clone(),
            iterable: self.iterable.clone(),
            context: None,
            other_filters: self.other_filters.clone(),
        }
    }
}

impl<P: MpsFilterPredicate + 'static> Display for MpsFilterStatement<P> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(other_filters) = &self.other_filters {
            write!(f, "{}.({} || (like) {})", self.iterable, self.predicate, other_filters)
        } else {
            write!(f, "{}.({})", self.iterable, self.predicate)
        }
    }
}

impl<P: MpsFilterPredicate + 'static> MpsOp for MpsFilterStatement<P> {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        let is_iterable_resetable = match &self.iterable {
            VariableOrOp::Variable(s) => {
                if self.context.is_some() {
                    let var = self.context.as_ref().unwrap().variables.get_opt(s);
                    if let Some(MpsType::Op(var)) = var {
                        var.is_resetable()
                    } else {
                        false
                    }
                } else {true} // ASSUMPTION

            }
            VariableOrOp::Op(PseudoOp::Real(op)) => op.is_resetable(),
            VariableOrOp::Op(PseudoOp::Fake(_)) => false,
        };
        let is_other_filter_resetable = if let Some(PseudoOp::Real(other_filter)) = &self.other_filters {
            other_filter.is_resetable()
        } else {true};
        is_iterable_resetable && is_other_filter_resetable
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        let fake = PseudoOp::Fake(format!("{}", self));
        self.predicate.reset()?;
        match &mut self.iterable {
            VariableOrOp::Variable(s) => {
                if self.context.as_mut().unwrap().variables.exists(s) {
                    let fake_getter = &mut move || fake.clone();
                    let mut var = self.context.as_mut().unwrap().variables.remove(s, fake_getter)?;
                    let result = if let MpsType::Op(var) = &mut var {
                        var.enter(self.context.take().unwrap());
                        let result = var.reset();
                        self.context = Some(var.escape());
                        result
                    } else {
                        Err(RuntimeError {
                            line: 0,
                            op: fake_getter(),
                            msg: "Cannot reset non-iterable filter variable".to_string(),
                        })
                    };
                    self.context.as_mut().unwrap().variables.declare(s, var, fake_getter)?;
                    result
                } else {Ok(())}
            },
            VariableOrOp::Op(PseudoOp::Real(op)) => {
                op.enter(self.context.take().unwrap());
                let result = op.reset();
                self.context = Some(op.escape());
                result
            },
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
        } else {Ok(())}
    }
}

impl<P: MpsFilterPredicate + 'static> Iterator for MpsFilterStatement<P> {
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.predicate.is_complete() && self.other_filters.is_none() {
            return None;
        }
        let self_clone = self.clone();
        let self_clone2 = self_clone.clone();
        let mut op_getter = move || (Box::new(self_clone.clone()) as Box<dyn MpsOp>).into();
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
                            },
                            Ok(item) => {
                                let matches_result =
                                    self.predicate.matches(&item, &mut ctx, &mut op_getter);
                                let matches = match matches_result {
                                    Err(e) => {
                                        maybe_result = Some(Err(e));
                                        self.context = Some(ctx);
                                        break;
                                    }
                                    Ok(b) => b,
                                };
                                if let Some(inner) = &mut self.other_filters {
                                    // handle other filters
                                    // make fake inner item
                                    let single_op = SingleItem::new_ok(item.clone());
                                    match ctx.variables.declare(INNER_VARIABLE_NAME, MpsType::Op(Box::new(single_op)), &mut op_getter) {
                                        Ok(x) => x,
                                        Err(e) => {
                                            //self.context = Some(op.escape());
                                            maybe_result = Some(Err(e));
                                            self.context = Some(ctx);
                                            break;
                                        },
                                    };
                                    let inner_real = match inner.try_real() {
                                        Ok(x) => x,
                                        Err(e) => {
                                            //self.context = Some(op.escape());
                                            maybe_result = Some(Err(e));
                                            self.context = Some(ctx);
                                            break;
                                        },
                                    };
                                    inner_real.enter(ctx);
                                    match inner_real.next() {
                                        Some(item) => {
                                            maybe_result = Some(item);
                                            ctx = inner_real.escape();
                                            match ctx.variables.remove(INNER_VARIABLE_NAME, &mut op_getter) {
                                                Ok(_) => {},
                                                Err(e) => match maybe_result {
                                                    Some(Ok(_)) => maybe_result = Some(Err(e)),
                                                    Some(Err(e2)) => maybe_result = Some(Err(e2)), // already failing, do not replace error,
                                                    None => {}, // impossible
                                                }
                                            }
                                            self.context = Some(ctx);
                                            break;
                                        },
                                        None => {
                                            ctx = inner_real.escape(); // move ctx back to expected spot
                                            match ctx.variables.remove(INNER_VARIABLE_NAME, &mut op_getter) {
                                                Ok(_) => {},
                                                Err(e) => {
                                                    //self.context = Some(op.escape());
                                                    maybe_result = Some(Err(e));
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
                Err(e) => return Some(Err(e)),
            },
            VariableOrOp::Variable(variable_name) => {
                let mut variable = match self
                    .context
                    .as_mut()
                    .unwrap()
                    .variables
                    .remove(&variable_name, &mut op_getter)
                {
                    Ok(MpsType::Op(op)) => op,
                    Ok(x) => {
                        return Some(Err(RuntimeError {
                            line: 0,
                            op: (Box::new(self_clone2.clone()) as Box<dyn MpsOp>).into(),
                            msg: format!(
                                "Expected operation/iterable type in variable {}, got {}",
                                &variable_name, x
                            ),
                        }))
                    }
                    Err(e) => return Some(Err(e)),
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
                            let matches_result =
                                self.predicate.matches(&item, &mut ctx, &mut op_getter);
                            let matches = match matches_result {
                                Err(e) => {
                                    maybe_result = Some(Err(e));
                                    self.context = Some(ctx);
                                    break;
                                }
                                Ok(b) => b,
                            };
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
                match self.context.as_mut().unwrap().variables.declare(
                    &variable_name,
                    MpsType::Op(variable),
                    &mut op_getter,
                ) {
                    Err(e) => return Some(Err(e)),
                    Ok(_) => maybe_result,
                }
            }
        }
    }
}

pub struct MpsFilterStatementFactory<
    P: MpsFilterPredicate + 'static,
    F: MpsFilterFactory<P> + 'static,
> {
    filter_factory: F,
    idc: PhantomData<P>,
}

impl<P: MpsFilterPredicate + 'static, F: MpsFilterFactory<P> + 'static>
    MpsFilterStatementFactory<P, F>
{
    pub fn new(factory: F) -> Self {
        Self {
            filter_factory: factory,
            idc: PhantomData::<P>,
        }
    }
}

impl<P: MpsFilterPredicate + 'static, F: MpsFilterFactory<P> + 'static> BoxedMpsOpFactory
    for MpsFilterStatementFactory<P, F>
{
    fn is_op_boxed(&self, tokens: &VecDeque<MpsToken>) -> bool {
        let tokens_len = tokens.len();
        if last_open_bracket_is_after_dot(tokens) {
            let start_of_predicate = last_dot_before_open_bracket(tokens) + 2; // .(predicate)
            if start_of_predicate > tokens_len - 1 {
                false
            } else {
                let pipe_location_opt = last_double_pipe(tokens, 1);
                if pipe_location_opt.is_some() && pipe_location_opt.unwrap() > start_of_predicate {
                    let pipe_location = pipe_location_opt.unwrap();
                    // filters combined by OR operations
                    let tokens2: VecDeque<&MpsToken> =
                        VecDeque::from_iter(tokens.range(start_of_predicate..pipe_location));
                    self.filter_factory.is_filter(&tokens2)
                } else {
                    // single filter
                    let tokens2: VecDeque<&MpsToken> =
                        VecDeque::from_iter(tokens.range(start_of_predicate..tokens_len - 1));
                    if tokens2.len() != 0 && check_name("if", &tokens2[0]) {
                        // replacement filter
                        if let Some(colon_location) = first_colon2(&tokens2) {
                            let tokens3 = VecDeque::from_iter(tokens.range(start_of_predicate+1..start_of_predicate+colon_location));
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
        tokens: &mut VecDeque<MpsToken>,
        dict: &MpsLanguageDictionary,
    ) -> Result<Box<dyn MpsOp>, SyntaxError> {
        let start_of_op = last_dot_before_open_bracket(tokens);
        let op;
        if start_of_op == 1 && tokens[0].is_name() {
            // variable_name.(predicate)
            let variable_name = assert_token(
                |t| match t {
                    MpsToken::Name(s) => Some(s),
                    _ => None,
                },
                MpsToken::Name("variable_name".into()),
                tokens,
            )?;
            op = VariableOrOp::Variable(variable_name);
        } else {
            // <some other op>.(predicate)
            //let mut new_tokens = tokens.range(0..start_of_op).map(|x| x.to_owned()).collect();
            let end_tokens = tokens.split_off(start_of_op); // don't parse filter in inner statement
            let inner_op = dict.try_build_statement(tokens)?;
            tokens.extend(end_tokens);
            op = VariableOrOp::Op(inner_op.into());
        }
        assert_token_raw(MpsToken::Dot, tokens)?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        if !tokens.is_empty() && check_name("if", &tokens[0]) {
            return {
                // replacement filter
                //println!("Building replacement filter from tokens {:?}", tokens);
                assert_name("if", tokens)?;
                if let Some(colon_location) = first_colon(tokens) {
                    let end_tokens = tokens.split_off(colon_location);
                    let filter = self.filter_factory.build_filter(tokens, dict)?;
                    tokens.extend(end_tokens);
                    assert_token_raw(MpsToken::Colon, tokens)?;
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
                    assert_token_raw(MpsToken::CloseBracket, tokens)?;
                    Ok(Box::new(MpsFilterReplaceStatement {
                        predicate: filter,
                        iterable: op,
                        context: None,
                        op_if: if_op,
                        op_else: else_op,
                        item_cache: super::filter_replace::item_cache_deque()
                    }))
                } else {
                    Err(SyntaxError {
                        line: 0,
                        token: MpsToken::Colon,
                        got: None,
                    })
                }
            }
        } else {
            let mut another_filter = None;
            let (has_or, end_tokens) = if let Some(pipe_location) = last_double_pipe(tokens, 1) {
                (true, tokens.split_off(pipe_location)) // parse up to OR operator
            } else {
                (false, tokens.split_off(tokens.len()-1)) // don't parse closing bracket in filter
            };
            let filter = self.filter_factory.build_filter(tokens, dict)?;
            tokens.extend(end_tokens);
            if has_or {
                // recursively build other filters for OR operation
                assert_token_raw(MpsToken::Pipe, tokens)?;
                assert_token_raw(MpsToken::Pipe, tokens)?;
                // emit fake filter syntax
                tokens.push_front(MpsToken::OpenBracket);
                tokens.push_front(MpsToken::Dot);
                tokens.push_front(MpsToken::Name(INNER_VARIABLE_NAME.into())); // impossible to obtain through parsing on purpose
                another_filter = Some(dict.try_build_statement(tokens)?.into());
            } else {
                assert_token_raw(MpsToken::CloseBracket, tokens)?; // remove closing bracket
            }
            Ok(Box::new(MpsFilterStatement {
                predicate: filter,
                iterable: op,
                context: None,
                other_filters: another_filter,
            }))
        }

    }
}

fn last_open_bracket_is_after_dot(tokens: &VecDeque<MpsToken>) -> bool {
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
                return true;
            } else {
                return false;
            }
        }
    }
    false
}

fn last_dot_before_open_bracket(tokens: &VecDeque<MpsToken>) -> usize {
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

fn last_double_pipe(tokens: &VecDeque<MpsToken>, in_brackets: usize) -> Option<usize> {
    let mut inside_brackets = 0;
    let mut pipe_found = false;
    for i in (0..tokens.len()).rev() {
        if tokens[i].is_pipe() && inside_brackets == in_brackets {
            if pipe_found {
                return Some(i);
            } else {
                pipe_found = true;
            }
        } else {
            pipe_found = false;
            if tokens[i].is_close_bracket() {
                inside_brackets += 1;
            } else if tokens[i].is_open_bracket() && inside_brackets != 0 {
                inside_brackets -= 1;
            }
        }
    }
    None
}

fn first_colon(tokens: &VecDeque<MpsToken>) -> Option<usize> {
    for i in 0..tokens.len() {
        if tokens[i].is_colon() {
            return Some(i);
        }
    }
    None
}

fn first_colon2(tokens: &VecDeque<&MpsToken>) -> Option<usize> {
    for i in 0..tokens.len() {
        if tokens[i].is_colon() {
            return Some(i);
        }
    }
    None
}

fn first_else_not_in_bracket(tokens: &VecDeque<MpsToken>) -> Option<usize> {
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
