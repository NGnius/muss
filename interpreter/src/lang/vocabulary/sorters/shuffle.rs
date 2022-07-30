use std::collections::VecDeque;
use std::fmt::{Debug, Display, Error, Formatter};

use rand::{thread_rng, Rng};

use crate::lang::utility::{assert_name, check_name};
use crate::lang::{IteratorItem, LanguageDictionary, Op};
use crate::lang::{RuntimeMsg, SyntaxError};
use crate::lang::{SortStatementFactory, Sorter, SorterFactory};
use crate::tokens::Token;

const RNG_LIMIT_BITMASK: usize = 0xffff; // bits to preserve in RNG
                                         // imposes an upper limit in the name of optimisation which reduces randomness past this point
                                         // this is also an effective item_buf size limit, 2^16 - 1 seems reasonable

#[derive(Debug, Clone)]
pub struct ShuffleSorter;

impl Sorter for ShuffleSorter {
    fn sort(
        &mut self,
        iterator: &mut dyn Op,
        item_buf: &mut VecDeque<IteratorItem>,
    ) -> Result<(), RuntimeMsg> {
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
            if item_buf.is_empty() {
                return Ok(());
            }
            random %= item_buf.len();
        }
    }
}

impl Display for ShuffleSorter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "random shuffle")
    }
}

pub struct ShuffleSorterFactory;

impl SorterFactory<ShuffleSorter> for ShuffleSorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&Token>) -> bool {
        (!tokens.is_empty() && check_name("shuffle", tokens[0]))
            || (tokens.len() > 1
                && check_name("random", tokens[0])
                && check_name("shuffle", tokens[1]))
    }

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<Token>,
        _dict: &LanguageDictionary,
    ) -> Result<ShuffleSorter, SyntaxError> {
        if check_name("random", &tokens[0]) {
            assert_name("random", tokens)?;
        }
        assert_name("shuffle", tokens)?;
        Ok(ShuffleSorter)
    }
}

pub type ShuffleSorterStatementFactory = SortStatementFactory<ShuffleSorter, ShuffleSorterFactory>;

#[inline(always)]
pub fn shuffle_sort() -> ShuffleSorterStatementFactory {
    ShuffleSorterStatementFactory::new(ShuffleSorterFactory)
}
