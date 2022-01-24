use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};
use std::cmp::Ordering;

use crate::lang::{MpsSorter, MpsSorterFactory, MpsSortStatementFactory};
use crate::lang::{MpsLanguageDictionary, MpsIteratorItem, MpsOp};
use crate::lang::{RuntimeError, SyntaxError};
use crate::lang::utility::assert_token;
use crate::tokens::MpsToken;

#[derive(Debug, Clone)]
pub struct FieldSorter {
    field_name: String,
    up_to: usize,
    default_order: Ordering,
}

impl MpsSorter for FieldSorter {
    fn sort(&mut self, iterator: &mut dyn MpsOp, item_buf: &mut VecDeque<MpsIteratorItem>) -> Result<(), RuntimeError> {
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
            item_buf.make_contiguous().sort_by(
                |a, b| {
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
                }
            );
            println!("Field-sorted item_buf: {:?}", item_buf);
        }
        Ok(())
    }
}

impl Display for FieldSorter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.field_name)
    }
}

pub struct FieldSorterFactory;

impl MpsSorterFactory<FieldSorter> for FieldSorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        tokens.len() == 1 && tokens[0].is_name()
    }

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<FieldSorter, SyntaxError> {
        let name = assert_token(|t| match t {
            MpsToken::Name(s) => Some(s),
            _ => None
        }, MpsToken::Name("field_name".into()), tokens)?;
        Ok(FieldSorter {
            field_name: name,
            up_to: usize::MAX,
            default_order: Ordering::Equal
        })
    }
}

pub type FieldSorterStatementFactory = MpsSortStatementFactory<FieldSorter, FieldSorterFactory>;

#[inline(always)]
pub fn field_sort() -> FieldSorterStatementFactory {
    FieldSorterStatementFactory::new(FieldSorterFactory)
}
