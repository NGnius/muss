use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_token, assert_token_raw};
use crate::lang::{IteratorItem, LanguageDictionary, Op};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{SortStatementFactory, Sorter, SorterFactory};
use crate::tokens::Token;

#[derive(Debug, Clone)]
pub struct FieldSorter {
    field_name: String,
    up_to: usize,
    default_order: Ordering,
}

impl Sorter for FieldSorter {
    fn sort(
        &mut self,
        iterator: &mut dyn Op,
        item_buf: &mut VecDeque<IteratorItem>,
    ) -> Result<(), RuntimeMsg> {
        let buf_len_old = item_buf.len(); // save buffer length before modifying buffer
        if item_buf.len() < self.up_to {
            for item in iterator {
                item_buf.push_back(item);
                if item_buf.len() >= self.up_to {
                    break;
                }
            }
        }
        if buf_len_old != item_buf.len() {
            // when buf_len_old == item_buf.len(), iterator was already complete
            // no need to sort in that case, since buffer was sorted in last call to sort or buffer never had any items to sort
            item_buf.make_contiguous().sort_by(|a, b| {
                if let Ok(a) = a {
                    if let Some(a_field) = a.field(&self.field_name) {
                        if let Ok(b) = b {
                            if let Some(b_field) = b.field(&self.field_name) {
                                return a_field.partial_cmp(b_field).unwrap_or(self.default_order);
                            }
                        }
                    }
                }
                self.default_order
            });
        }
        Ok(())
    }
}

impl Display for FieldSorter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, ".{}", self.field_name)
    }
}

pub struct FieldSorterFactory;

impl SorterFactory<FieldSorter> for FieldSorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&Token>) -> bool {
        !tokens.is_empty() && tokens[0].is_dot()
    }

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<FieldSorter, SyntaxError> {
        assert_token_raw(Token::Dot, tokens)?;
        let name = assert_token(
            |t| match t {
                Token::Name(s) => Some(s),
                _ => None,
            },
            Token::Name("field_name".into()),
            tokens,
        )?;
        Ok(FieldSorter {
            field_name: name,
            up_to: usize::MAX,
            default_order: Ordering::Greater,
        })
    }
}

pub type FieldSorterStatementFactory = SortStatementFactory<FieldSorter, FieldSorterFactory>;

#[inline(always)]
pub fn field_sort() -> FieldSorterStatementFactory {
    FieldSorterStatementFactory::new(FieldSorterFactory)
}
