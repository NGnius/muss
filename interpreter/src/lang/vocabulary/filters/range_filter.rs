use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::assert_token_raw;
use crate::lang::Lookup;
use crate::lang::{FilterFactory, FilterPredicate, FilterStatementFactory};
use crate::lang::{LanguageDictionary, TypePrimitive};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::processing::general::Type;
use crate::tokens::Token;
use crate::Context;
use crate::Item;

#[derive(Debug, Clone)]
pub struct RangeFilter {
    start: Option<Lookup>,
    end: Option<Lookup>,
    inclusive_end: bool,
    // state
    current: u64,
    complete: bool,
}

impl Display for RangeFilter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}{}{}",
            if let Some(start) = &self.start {
                format!("{}", start)
            } else {
                "".into()
            },
            if self.inclusive_end { "=" } else { "" },
            if let Some(end) = &self.end {
                format!("{}", end)
            } else {
                "".into()
            },
        )
    }
}

impl FilterPredicate for RangeFilter {
    fn matches(&mut self, _item: &Item, ctx: &mut Context) -> Result<bool, RuntimeMsg> {
        let start_index = if let Some(start) = &self.start {
            lookup_to_index(start.get(ctx)?)?
        } else {
            0
        };
        let current = self.current;
        self.current += 1;
        if current >= start_index {
            if let Some(end) = &self.end {
                let end_index = lookup_to_index(end.get(ctx)?)?;
                if self.inclusive_end && current <= end_index {
                    if current == end_index {
                        self.complete = true;
                    }
                    Ok(true)
                } else if !self.inclusive_end && current < end_index {
                    if self.current == end_index {
                        self.complete = true;
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn reset(&mut self) -> Result<(), RuntimeMsg> {
        self.current = 0;
        self.complete = false;
        Ok(())
    }
}

fn lookup_to_index(item: &Type) -> Result<u64, RuntimeMsg> {
    match item {
        Type::Primitive(val) => match val {
            TypePrimitive::Int(i) => Ok(*i as u64),
            TypePrimitive::UInt(u) => Ok(*u),
            TypePrimitive::Float(f) => Ok(*f as u64),
            val => Err(RuntimeMsg(format!("Cannot use {} as index", val))),
        },
        val => Err(RuntimeMsg(format!("Cannot use {} as index", val))),
    }
}

pub struct RangeFilterFactory;

impl FilterFactory<RangeFilter> for RangeFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&Token>) -> bool {
        tokens.len() > 1
            && ((tokens[0].is_dot() && tokens[1].is_dot())
                || (tokens.len() > 2
                    && Lookup::check_is(tokens[0])
                    && tokens[1].is_dot()
                    && tokens[2].is_dot()))
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<RangeFilter, SyntaxError> {
        // start index
        let start = if Lookup::check_is(&tokens[0]) {
            Some(Lookup::parse(tokens)?)
        } else {
            None
        };
        // ..
        assert_token_raw(Token::Dot, tokens)?;
        assert_token_raw(Token::Dot, tokens)?;
        // tokens VecDeque might now be empty (guaranteed to have tokens up to this point)
        // = (optional)
        let equals_at_end = if !tokens.is_empty() && tokens[0].is_equals() {
            assert_token_raw(Token::Equals, tokens)?;
            true
        } else {
            false
        };
        // end index
        let end = if !tokens.is_empty() && Lookup::check_is(&tokens[0]) {
            Some(Lookup::parse(tokens)?)
        } else {
            None
        };

        Ok(RangeFilter {
            start,
            end,
            inclusive_end: equals_at_end,
            current: 0,
            complete: false,
        })
    }
}

pub type RangeFilterStatementFactory = FilterStatementFactory<RangeFilter, RangeFilterFactory>;

#[inline(always)]
pub fn range_filter() -> RangeFilterStatementFactory {
    RangeFilterStatementFactory::new(RangeFilterFactory)
}
