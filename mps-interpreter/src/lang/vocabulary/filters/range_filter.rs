use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::{MpsLanguageDictionary, MpsTypePrimitive};
use crate::lang::{MpsFilterFactory, MpsFilterPredicate, MpsFilterStatementFactory};
use crate::lang::{RuntimeError, SyntaxError};
use crate::lang::Lookup;
use crate::lang::utility::assert_token_raw;
use crate::processing::{OpGetter, general::MpsType};
use crate::tokens::MpsToken;
use crate::MpsContext;
use crate::MpsMusicItem;

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
        write!(f, "{}{}{}",
            if let Some(start) = &self.start {format!("{}", start)} else {"".into()},
            if self.inclusive_end {"="} else {""},
            if let Some(end) = &self.end {format!("{}", end)} else {"".into()},)
    }
}

impl MpsFilterPredicate for RangeFilter {
    fn matches(
        &mut self,
        _item: &MpsMusicItem,
        ctx: &mut MpsContext,
        op: &mut OpGetter,
    ) -> Result<bool, RuntimeError> {
        let start_index = if let Some(start) = &self.start {
            lookup_to_index(start.get(ctx, op)?, op)?
        } else {0};
        let current = self.current;
        self.current += 1;
        if current >= start_index {
            if let Some(end) = &self.end {
                let end_index = lookup_to_index(end.get(ctx, op)?, op)?;
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

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.current = 0;
        self.complete = false;
        Ok(())
    }
}

fn lookup_to_index(item: &MpsType, op: &mut OpGetter) -> Result<u64, RuntimeError> {
    match item {
        MpsType::Primitive(val) => match val {
            MpsTypePrimitive::Int(i) => Ok(*i as u64),
            MpsTypePrimitive::UInt(u) => Ok(*u),
            MpsTypePrimitive::Float(f) => Ok(*f as u64),
            val => Err(RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Cannot use {} as index", val),
            })
        },
        val => Err(RuntimeError {
            line: 0,
            op: op(),
            msg: format!("Cannot use {} as index", val),
        })
    }
}

pub struct RangeFilterFactory;

impl MpsFilterFactory<RangeFilter> for RangeFilterFactory {
    fn is_filter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        (
            // ..
            tokens.len() == 2
            && tokens[0].is_dot()
            && tokens[1].is_dot()
        ) || (
            tokens.len() == 3
            && ((
                // ..number
                tokens[0].is_dot()
                && tokens[1].is_dot()
                && Lookup::check_is(&tokens[2])
            ) || (
                // number..
                Lookup::check_is(&tokens[0])
                && tokens[1].is_dot()
                && tokens[2].is_dot()
            ))
        ) || (
            tokens.len() == 4
            && (( // number..number
                Lookup::check_is(&tokens[0])
                && tokens[1].is_dot()
                && tokens[2].is_dot()
                && Lookup::check_is(&tokens[3])
            ) || ( // ..=number
                tokens[0].is_dot()
                && tokens[1].is_dot()
                && tokens[2].is_equals()
                && Lookup::check_is(&tokens[3])
            ))
        ) || (
            // number..=number
            tokens.len() == 5
            && Lookup::check_is(&tokens[0])
            && tokens[1].is_dot()
            && tokens[2].is_dot()
            && tokens[3].is_equals()
            && Lookup::check_is(&tokens[4])
        )
    }

    fn build_filter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<RangeFilter, SyntaxError> {
        // start index
        let start = if Lookup::check_is(&tokens[0]) {
            Some(Lookup::parse(tokens)?)
        } else {None};
        // ..
        assert_token_raw(MpsToken::Dot, tokens)?;
        assert_token_raw(MpsToken::Dot, tokens)?;
        // tokens VecDeque might now be empty (guaranteed to have tokens up to this point)
        // = (optional)
        let equals_at_end = if !tokens.is_empty() && tokens[0].is_equals() {
            assert_token_raw(MpsToken::Equals, tokens)?;
            true
        } else {
            false
        };
        // end index
        let end = if !tokens.is_empty() {
            Some(Lookup::parse(tokens)?)
        } else {None};

        Ok(RangeFilter {
            start: start,
            end: end,
            inclusive_end: equals_at_end,
            current: 0,
            complete: false,
        })
    }
}

pub type RangeFilterStatementFactory = MpsFilterStatementFactory<RangeFilter, RangeFilterFactory>;

#[inline(always)]
pub fn range_filter() -> RangeFilterStatementFactory {
    RangeFilterStatementFactory::new(RangeFilterFactory)
}
