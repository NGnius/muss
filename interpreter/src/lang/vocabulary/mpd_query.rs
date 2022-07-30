use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::net::SocketAddr;

use crate::tokens::Token;
use crate::Context;

use crate::lang::{LanguageDictionary, repeated_tokens, Lookup};
use crate::lang::{FunctionFactory, FunctionStatementFactory, IteratorItem, Op, PseudoOp};
use crate::lang::{RuntimeError, SyntaxError, RuntimeOp};
use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::TypePrimitive;
use crate::processing::general::Type;
use crate::Item;

#[cfg(feature = "mpd")]
#[derive(Debug)]
pub struct MpdQueryStatement {
    context: Option<Context>,
    addr: Lookup,
    params: Vec<(String, Lookup)>,
    results: Option<VecDeque<Item>>,
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
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        //let ctx = self.context.as_mut().unwrap();
        if self.results.is_none() {
            self.results = Some(VecDeque::with_capacity(0)); // in case of failure
            // build address
            let addr_str = match self.addr.get(self.context.as_mut().unwrap()) {
                Ok(Type::Primitive(a)) => a.as_str(),
                Ok(x) => return Some(Err(
                    RuntimeError {
                        line: 0,
                        msg: format!("Cannot use non-primitive `{}` as IP address", x),
                        op: PseudoOp::from_printable(self),
                    }
                )),
                Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            };
            #[cfg(not(feature = "ergonomics"))]
            let addr: SocketAddr = match addr_str.parse() {
                Ok(a) => a,
                Err(e) => return Some(Err(RuntimeError {
                    line: 0,
                    op: PseudoOp::from_printable(self),
                    msg: format!("Cannot convert `{}` to IP Address: {}", addr_str, e),
                }))
            };
            #[cfg(feature = "ergonomics")]
            let addr: SocketAddr = if addr_str.starts_with("localhost:") {
                let port_str = addr_str.replace("localhost:", "");
                let port = match port_str.parse::<u16>() {
                    Ok(p) => p,
                    Err(e) => return Some(Err(RuntimeError {
                        line: 0,
                        op: PseudoOp::from_printable(self),
                        msg: format!("Cannot convert `{}` to IP port: {}", port_str, e),
                    }))
                };
                SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, port))
            } else if addr_str == "default" {
                SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, 6600))
            } else {
                match addr_str.parse() {
                    Ok(a) => a,
                    Err(e) => return Some(Err(RuntimeError {
                        line: 0,
                        op: PseudoOp::from_printable(self),
                        msg: format!("Cannot convert `{}` to IP Address: {}", addr_str, e),
                    }))
                }
            };
            // build params
            let mut new_params = Vec::<(&str, String)>::with_capacity(self.params.len());
            for (term, value) in self.params.iter() {
                let static_val = match value.get(self.context.as_mut().unwrap()) {
                    Ok(Type::Primitive(a)) => a.as_str(),
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
        results.pop_front().map(Ok)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

#[cfg(feature = "mpd")]
impl Op for MpdQueryStatement {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(self.clone())
    }
}

#[cfg(feature = "mpd")]
pub struct MpdQueryFunctionFactory;

#[cfg(feature = "mpd")]
impl FunctionFactory<MpdQueryStatement> for MpdQueryFunctionFactory {
    fn is_function(&self, name: &str) -> bool {
        name == "mpd"
    }

    fn build_function_params(
        &self,
        _name: String,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<MpdQueryStatement, SyntaxError> {
        // mpd(address, term = value, ...)
        let addr_lookup = Lookup::parse(tokens)?;
        if tokens.is_empty() {
            Ok(MpdQueryStatement {
                context: None,
                addr: addr_lookup,
                params: vec![("any".to_string(), Lookup::Static(Type::Primitive(TypePrimitive::String("".to_owned()))))],
                results: None,
            })
        } else {
            assert_token_raw(Token::Comma, tokens)?;
            let keyword_params = repeated_tokens(
                |tokens| {
                    let term = assert_token(
                        |t| match t {
                            Token::Name(n) => Some(n),
                            _ => None,
                        },
                        Token::Name("term".to_string()),
                        tokens)?;
                    assert_token_raw(Token::Equals, tokens)?;
                    let val = Lookup::parse(tokens)?;
                    Ok(Some((term, val)))
                },
                Token::Comma
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
pub type MpdQueryStatementFactory = FunctionStatementFactory<MpdQueryStatement, MpdQueryFunctionFactory>;

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
