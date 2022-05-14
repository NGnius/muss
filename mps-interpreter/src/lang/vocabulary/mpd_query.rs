use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::net::SocketAddr;

use crate::tokens::MpsToken;
use crate::MpsContext;

use crate::lang::{MpsLanguageDictionary, repeated_tokens, Lookup};
use crate::lang::{MpsFunctionFactory, MpsFunctionStatementFactory, MpsIteratorItem, MpsOp, PseudoOp};
use crate::lang::{RuntimeError, SyntaxError, RuntimeOp};
use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::MpsTypePrimitive;
use crate::processing::general::MpsType;
use crate::MpsItem;

#[cfg(feature = "mpd")]
#[derive(Debug)]
pub struct MpdQueryStatement {
    context: Option<MpsContext>,
    addr: Lookup,
    params: Vec<(String, Lookup)>,
    results: Option<VecDeque<MpsItem>>,
}

#[cfg(feature = "mpd")]
impl Display for MpdQueryStatement {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "empty()")
    }
}

#[cfg(feature = "mpd")]
impl std::clone::Clone for MpdQueryStatement {
    fn clone(&self) -> Self {
        Self {
            context: None,
            addr: self.addr.clone(),
            params: self.params.clone(),
            results: None,
        }
    }
}

#[cfg(feature = "mpd")]
impl Iterator for MpdQueryStatement {
    type Item = MpsIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        //let ctx = self.context.as_mut().unwrap();
        if self.results.is_none() {
            self.results = Some(VecDeque::with_capacity(0)); // in case of failure
            // build address
            let addr_str = match self.addr.get(self.context.as_mut().unwrap()) {
                Ok(MpsType::Primitive(a)) => a.as_str(),
                Ok(x) => return Some(Err(
                    RuntimeError {
                        line: 0,
                        msg: format!("Cannot use non-primitive `{}` as IP address", x),
                        op: PseudoOp::from_printable(self),
                    }
                )),
                Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            };
            let addr: SocketAddr = match addr_str.parse() {
                Ok(a) => a,
                Err(e) => return Some(Err(RuntimeError {
                    line: 0,
                    op: PseudoOp::from_printable(self),
                    msg: format!("Cannot convert `{}` to IP Address: {}", addr_str, e),
                }))
            };
            // build params
            let mut new_params = Vec::<(&str, String)>::with_capacity(self.params.len());
            for (term, value) in self.params.iter() {
                let static_val = match value.get(self.context.as_mut().unwrap()) {
                    Ok(MpsType::Primitive(a)) => a.as_str(),
                    Ok(x) => return Some(Err(
                        RuntimeError {
                            line: 0,
                            msg: format!("Cannot use non-primitive `{}` MPS query value", x),
                            op: PseudoOp::from_printable(self),
                        }
                    )),
                    Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
                };
                new_params.push((term, static_val));
            }
            self.results = Some(match self.context.as_mut().unwrap().mpd_database.one_shot_search(addr, new_params) {
                Ok(items) => items,
                Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self)))))
            });
        }
        let results = self.results.as_mut().unwrap();
        results.pop_front().map(|x| Ok(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

#[cfg(feature = "mpd")]
impl MpsOp for MpdQueryStatement {
    fn enter(&mut self, ctx: MpsContext) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> MpsContext {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn dup(&self) -> Box<dyn MpsOp> {
        Box::new(self.clone())
    }
}

#[cfg(feature = "mpd")]
pub struct MpdQueryFunctionFactory;

#[cfg(feature = "mpd")]
impl MpsFunctionFactory<MpdQueryStatement> for MpdQueryFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "mpd"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<MpdQueryStatement, SyntaxError> {
        // mpd(address, term = value, ...)
        let addr_lookup = Lookup::parse(tokens)?;
        if tokens.is_empty() {
            Ok(MpdQueryStatement {
                context: None,
                addr: addr_lookup,
                params: vec![("any".to_string(), Lookup::Static(MpsType::Primitive(MpsTypePrimitive::String("".to_owned()))))],
                results: None,
            })
        } else {
            assert_token_raw(MpsToken::Comma, tokens)?;
            let keyword_params = repeated_tokens(
                |tokens| {
                    let term = assert_token(
                        |t| match t {
                            MpsToken::Name(n) => Some(n),
                            _ => None,
                        },
                        MpsToken::Name("term".to_string()),
                        tokens)?;
                    assert_token_raw(MpsToken::Equals, tokens)?;
                    let val = Lookup::parse(tokens)?;
                    Ok(Some((term, val)))
                },
                MpsToken::Comma
                ).ingest_all(tokens)?;
            Ok(MpdQueryStatement {
                context: None,
                addr: addr_lookup,
                params: keyword_params,
                results: None,
            })
        }
    }
}

#[cfg(feature = "mpd")]
pub type MpdQueryStatementFactory = MpsFunctionStatementFactory<MpdQueryStatement, MpdQueryFunctionFactory>;

#[cfg(feature = "mpd")]
#[inline(always)]
pub fn mpd_query_function_factory() -> MpdQueryStatementFactory {
    MpdQueryStatementFactory::new(MpdQueryFunctionFactory)
}

#[cfg(not(feature = "mpd"))]
pub type MpdQueryStatementFactory = super::EmptyStatementFactory;

#[cfg(not(feature = "mpd"))]
#[inline(always)]
pub fn mpd_query_function_factory() -> MpdQueryStatementFactory {
    super::empty_function_factory()
}
