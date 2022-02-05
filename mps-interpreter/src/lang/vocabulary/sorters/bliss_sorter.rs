use std::collections::VecDeque;
#[cfg(feature = "bliss-audio")]
use std::fmt::{Debug, Display, Error, Formatter};

#[cfg(feature = "bliss-audio")]
use std::collections::HashMap;

use crate::lang::utility::{assert_name, check_name};
use crate::lang::SyntaxError;
#[cfg(feature = "bliss-audio")]
use crate::lang::{MpsIteratorItem, MpsOp, MpsSorter, RuntimeMsg};
use crate::lang::{MpsLanguageDictionary, MpsSortStatementFactory, MpsSorterFactory};
use crate::tokens::MpsToken;
#[cfg(feature = "bliss-audio")]
use crate::MpsItem;

#[cfg(feature = "bliss-audio")]
const DEFAULT_ORDER: std::cmp::Ordering = std::cmp::Ordering::Greater;

#[cfg(feature = "bliss-audio")]
#[derive(Debug)]
pub struct BlissSorter {
    up_to: usize,
    first_song: Option<MpsItem>,
}

#[cfg(feature = "bliss-audio")]
impl std::clone::Clone for BlissSorter {
    fn clone(&self) -> Self {
        Self {
            up_to: self.up_to,
            first_song: self.first_song.clone(),
        }
    }
}

#[cfg(feature = "bliss-audio")]
impl Default for BlissSorter {
    fn default() -> Self {
        Self {
            up_to: usize::MAX,
            first_song: None,
        }
    }
}

#[cfg(feature = "bliss-audio")]
impl MpsSorter for BlissSorter {
    fn sort(
        &mut self,
        iterator: &mut dyn MpsOp,
        item_buf: &mut VecDeque<MpsIteratorItem>,
    ) -> Result<(), RuntimeMsg> {
        let buf_len_old = item_buf.len(); // save buffer length before modifying buffer
        if item_buf.len() < self.up_to {
            while let Some(item) = iterator.next() {
                item_buf.push_back(item);
                if item_buf.len() >= self.up_to {
                    break;
                }
            }
        }
        if buf_len_old != item_buf.len() && !item_buf.is_empty() {
            // when buf_len_old == item_buf.len(), iterator was already complete
            // no need to sort in that case, since buffer was sorted in last call to sort or buffer never had any items to sort
            if self.first_song.is_none() {
                for item in item_buf.iter() {
                    if let Ok(item) = item {
                        self.first_song = Some(item.clone());
                        break;
                    }
                }
            }
            if let Some(first) = &self.first_song {
                let mut ctx = iterator.escape();
                for i in 0..item_buf.len() {
                    if let Ok(item) = &item_buf[i] {
                        if item == first {continue;}
                        match ctx.analysis.prepare_distance(first, item) {
                            Err(e) => {
                                iterator.enter(ctx);
                                return Err(e);
                            },
                            Ok(_) => {},
                        }
                    }
                }
                iterator.enter(ctx);
            }

        } else if self.first_song.is_some() {
            // Sort songs on second call to this function
            let first = self.first_song.take().unwrap();
            let mut cache = HashMap::<MpsItem, f64>::new();
            cache.insert(first.clone(), 0.0);
            let mut ctx = iterator.escape();
            for i in 0..item_buf.len() {
                if let Ok(item) = &item_buf[i] {
                    if item == &first {continue;}
                    match ctx.analysis.get_distance(&first, item) {
                        Err(e) => {
                            iterator.enter(ctx);
                            return Err(e);
                        },
                        Ok(distance) => {
                            cache.insert(item.clone(), distance);
                        },
                    }
                }
            }
            iterator.enter(ctx);
            item_buf.make_contiguous().sort_by(|a, b| {
                if let Ok(a) = a {
                    if let Ok(b) = b {
                        let float_a = cache.get(&a).unwrap();
                        let float_b = cache.get(&b).unwrap();
                        return float_a.partial_cmp(float_b).unwrap_or(DEFAULT_ORDER);
                    }
                }
                DEFAULT_ORDER
            });
        }
        Ok(())
    }
}

#[cfg(not(feature = "bliss-audio"))]
pub type BlissSorter = crate::lang::vocabulary::sorters::EmptySorter;

#[cfg(feature = "bliss-audio")]
impl Display for BlissSorter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "advanced bliss_first")
    }
}

pub struct BlissSorterFactory;

impl MpsSorterFactory<BlissSorter> for BlissSorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        tokens.len() == 2
            && check_name("advanced", tokens[0])
            && check_name("bliss_first", tokens[1])
    }

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<BlissSorter, SyntaxError> {
        assert_name("advanced", tokens)?;
        assert_name("bliss_first", tokens)?;
        Ok(BlissSorter::default())
    }
}

pub type BlissSorterStatementFactory = MpsSortStatementFactory<BlissSorter, BlissSorterFactory>;

#[inline(always)]
pub fn bliss_sort() -> BlissSorterStatementFactory {
    BlissSorterStatementFactory::new(BlissSorterFactory)
}
