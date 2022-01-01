use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::marker::PhantomData;

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::MpsLanguageDictionary;
use crate::lang::{BoxedMpsOpFactory, MpsOp, PseudoOp};
use crate::lang::{RuntimeError, SyntaxError};
use crate::processing::general::MpsType;
use crate::processing::OpGetter;
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsMusicItem;

pub trait MpsFilterPredicate: Clone + Debug + Display {
    fn matches(
        &mut self,
        item: &MpsMusicItem,
        ctx: &mut MpsContext,
        op: &mut OpGetter,
    ) -> Result<bool, RuntimeError>;
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
enum VariableOrOp {
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
}

impl<P: MpsFilterPredicate + 'static> std::clone::Clone for MpsFilterStatement<P> {
    fn clone(&self) -> Self {
        Self {
            predicate: self.predicate.clone(),
            iterable: self.iterable.clone(),
            context: None,
        }
    }
}

impl<P: MpsFilterPredicate + 'static> Display for MpsFilterStatement<P> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}.({})", self.iterable, self.predicate)
    }
}

impl<P: MpsFilterPredicate + 'static> MpsOp for MpsFilterStatement<P> {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }
}

impl<P: MpsFilterPredicate + 'static> Iterator for MpsFilterStatement<P> {
    type Item = Result<MpsMusicItem, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
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
            idc: PhantomData::default(),
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
                let tokens2: VecDeque<&MpsToken> =
                    VecDeque::from_iter(tokens.range(start_of_predicate..tokens_len - 1));
                self.filter_factory.is_filter(&tokens2)
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
            let end_tokens = tokens.split_off(start_of_op);
            let inner_op = dict.try_build_statement(tokens)?;
            tokens.extend(end_tokens);
            op = VariableOrOp::Op(inner_op.into());
        }
        assert_token_raw(MpsToken::Dot, tokens)?;
        assert_token_raw(MpsToken::OpenBracket, tokens)?;
        let filter = self.filter_factory.build_filter(tokens, dict)?;
        assert_token_raw(MpsToken::CloseBracket, tokens)?;
        Ok(Box::new(MpsFilterStatement {
            predicate: filter,
            iterable: op,
            context: None,
        }))
    }
}

fn last_open_bracket_is_after_dot(tokens: &VecDeque<MpsToken>) -> bool {
    let mut open_bracket_found = false;
    for i in (0..tokens.len()).rev() {
        if tokens[i].is_open_bracket() {
            open_bracket_found = true;
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
    let mut open_bracket_found = false;
    for i in (0..tokens.len()).rev() {
        if tokens[i].is_open_bracket() {
            open_bracket_found = true;
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
