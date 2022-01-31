use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use rand::{thread_rng, Rng};

use crate::lang::{MpsIteratorItem, MpsLanguageDictionary, MpsOp};
use crate::lang::{MpsSortStatementFactory, MpsSorter, MpsSorterFactory};
use crate::lang::{RuntimeError, SyntaxError};
use crate::lang::utility::{check_name, assert_name};
use crate::processing::OpGetter;
use crate::tokens::MpsToken;

const RNG_LIMIT_BITMASK: usize = 0xffff; // bits to preserve in RNG
// imposes an upper limit in the name of optimisation which reduces randomness past this point
// this is also an effective item_buf size limit, 2^16 - 1 seems reasonable

#[derive(Debug, Clone)]
pub struct ShuffleSorter;

impl MpsSorter for ShuffleSorter {
    fn sort(
        &mut self,
        iterator: &mut dyn MpsOp,
        item_buf: &mut VecDeque<MpsIteratorItem>,
        _op: &mut OpGetter,
    ) -> Result<(), RuntimeError> {
        // iterative shuffling algorithm
        //
        // choose a random number r
        // loop:
        //   if buffer length > r: return buffer[r] (removing buffer[r])
        //   else:
        //     traverse iterator until r - buffer length is encountered
        //     fill buffer with items as it passes
        //     if end of iterator encountered: r = r % buffer length, repeat loop
        //     else: return iterator item
        //
        // the following is similar, except using VecDeque.swap_remove_back() to avoid large memory moves
        let r: usize = thread_rng().gen();
        let mut random: usize = r & RNG_LIMIT_BITMASK;
        loop {
            if item_buf.len() > random {
                let item = item_buf.swap_remove_back(random).unwrap();
                item_buf.push_front(item);
                return Ok(());
            }
            let mut iterator_pos = item_buf.len();
            while let Some(item) = iterator.next() {
                if iterator_pos == random {
                    item_buf.push_front(item);
                    return Ok(());
                } else {
                    iterator_pos += 1;
                    item_buf.push_back(item);
                }
            }
            // end case: everything is completely empty -- end loop without a new result
            if item_buf.len() == 0 {
                return Ok(());
            }
            random = random % item_buf.len();
        }
    }
}

impl Display for ShuffleSorter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "random shuffle")
    }
}

pub struct ShuffleSorterFactory;

impl MpsSorterFactory<ShuffleSorter> for ShuffleSorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        (tokens.len() == 1 && check_name("shuffle", &tokens[0]))
        ||
        (tokens.len() == 2 && check_name("random", &tokens[0]) && check_name("shuffle", &tokens[1]))
    }

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<ShuffleSorter, SyntaxError> {
        if check_name("random", &tokens[0]) {
            assert_name("random", tokens)?;
        }
        assert_name("shuffle", tokens)?;
        Ok(ShuffleSorter)
    }
}

pub type ShuffleSorterStatementFactory = MpsSortStatementFactory<ShuffleSorter, ShuffleSorterFactory>;

#[inline(always)]
pub fn shuffle_sort() -> ShuffleSorterStatementFactory {
    ShuffleSorterStatementFactory::new(ShuffleSorterFactory)
}
