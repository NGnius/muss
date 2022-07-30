use std::collections::VecDeque;
#[cfg(feature = "advanced")]
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::utility::{assert_name, check_name};
use crate::lang::SyntaxError;
#[cfg(feature = "advanced")]
use crate::lang::{IteratorItem, Op, RuntimeMsg, Sorter};
use crate::lang::{LanguageDictionary, SortStatementFactory, SorterFactory};
use crate::tokens::Token;
#[cfg(feature = "advanced")]
use crate::Item;

#[cfg(feature = "advanced")]
#[derive(Debug)]
pub struct BlissNextSorter {
    up_to: usize,
    algorithm_done: bool,
    init_done: bool,
    item_buf: VecDeque<Item>,
}

#[cfg(feature = "advanced")]
impl std::clone::Clone for BlissNextSorter {
    fn clone(&self) -> Self {
        Self {
            up_to: self.up_to,
            algorithm_done: self.algorithm_done,
            init_done: self.init_done,
            item_buf: self.item_buf.clone(),
        }
    }
}

#[cfg(feature = "advanced")]
impl Default for BlissNextSorter {
    fn default() -> Self {
        Self {
            up_to: usize::MAX,
            algorithm_done: false,
            init_done: false,
            item_buf: VecDeque::new(),
        }
    }
}

#[cfg(feature = "advanced")]
impl Sorter for BlissNextSorter {
    fn sort(
        &mut self,
        iterator: &mut dyn Op,
        items_out: &mut VecDeque<IteratorItem>,
    ) -> Result<(), RuntimeMsg> {
        if !self.init_done {
            // first run
            self.init_done = true;
            while let Some(item) = iterator.next() {
                match item {
                    Ok(item) => self.item_buf.push_back(item),
                    Err(e) => items_out.push_back(Err(e)),
                }
                if self.item_buf.len() + items_out.len() >= self.up_to {
                    break;
                }
            }
            if !self.item_buf.is_empty() {
                let first = &self.item_buf[0];
                let mut ctx = iterator.escape();
                for i in 1..self.item_buf.len() {
                    let item = &self.item_buf[i];
                    if let Err(e) = ctx.analysis.prepare_distance(first, item) {
                        iterator.enter(ctx);
                        return Err(e);
                    }
                }
                iterator.enter(ctx);
                items_out.push_back(Ok(first.to_owned()));
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if self.item_buf.len() > 2 {
                let last = self.item_buf.pop_front().unwrap();
                let mut best_index = 0;
                let mut best_distance = f64::MAX;
                let mut ctx = iterator.escape();
                for i in 0..self.item_buf.len() {
                    let current_item = &self.item_buf[i];
                    match ctx.analysis.get_distance(&last, current_item) {
                        Err(e) => {
                            iterator.enter(ctx);
                            return Err(e);
                        }
                        Ok(distance) => {
                            if distance < best_distance {
                                best_index = i;
                                best_distance = distance;
                            }
                        }
                    }
                }
                if best_index != 0 {
                    self.item_buf.swap(0, best_index);
                }
                items_out.push_back(Ok(self.item_buf[0].clone()));
                let next = &self.item_buf[0];
                for i in 1..self.item_buf.len() {
                    let item = &self.item_buf[i];
                    if let Err(e) = ctx.analysis.prepare_distance(next, item) {
                        iterator.enter(ctx);
                        return Err(e);
                    }
                }
                iterator.enter(ctx);
            } else if self.item_buf.len() == 2 {
                self.item_buf.pop_front();
                items_out.push_back(Ok(self.item_buf.pop_front().unwrap()));
                // note item_buf is emptied here, so this will not proceed to len() == 1 case on next call
            } else if !self.item_buf.is_empty() {
                // edge case where item_buf only ever had 1 item
                items_out.push_back(Ok(self.item_buf.pop_front().unwrap()));
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.init_done = false;
    }
}

#[cfg(not(feature = "advanced"))]
pub type BlissNextSorter = crate::lang::vocabulary::sorters::EmptySorter;

#[cfg(feature = "advanced")]
impl Display for BlissNextSorter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "advanced bliss_next")
    }
}

pub struct BlissNextSorterFactory;

impl SorterFactory<BlissNextSorter> for BlissNextSorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&Token>) -> bool {
        tokens.len() > 1 && check_name("advanced", tokens[0]) && check_name("bliss_next", tokens[1])
    }

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<BlissNextSorter, SyntaxError> {
        assert_name("advanced", tokens)?;
        assert_name("bliss_next", tokens)?;
        Ok(BlissNextSorter::default())
    }
}

pub type BlissNextSorterStatementFactory =
    SortStatementFactory<BlissNextSorter, BlissNextSorterFactory>;

#[inline(always)]
pub fn bliss_next_sort() -> BlissNextSorterStatementFactory {
    BlissNextSorterStatementFactory::new(BlissNextSorterFactory)
}
