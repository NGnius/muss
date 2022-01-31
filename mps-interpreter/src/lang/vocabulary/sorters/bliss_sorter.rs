use std::collections::VecDeque;
#[cfg(feature = "bliss-audio")]
use std::fmt::{Debug, Display, Error, Formatter};
#[cfg(feature = "bliss-audio")]
use std::sync::mpsc::{channel, Receiver};

#[cfg(feature = "bliss-audio")]
use std::collections::HashMap;

#[cfg(feature = "bliss-audio")]
use bliss_audio::Song;

use crate::lang::utility::{assert_name, check_name};
use crate::lang::SyntaxError;
#[cfg(feature = "bliss-audio")]
use crate::lang::{MpsIteratorItem, MpsOp, MpsSorter, MpsTypePrimitive, RuntimeError};
use crate::lang::{MpsLanguageDictionary, MpsSortStatementFactory, MpsSorterFactory};
#[cfg(feature = "bliss-audio")]
use crate::processing::OpGetter;
use crate::tokens::MpsToken;

#[cfg(feature = "bliss-audio")]
const DEFAULT_ORDER: std::cmp::Ordering = std::cmp::Ordering::Greater;

#[cfg(feature = "bliss-audio")]
#[derive(Debug)]
pub struct BlissSorter {
    up_to: usize,
    float_map: HashMap<String, f32>,
    first_song: Option<String>,
    rx: Option<Receiver<Result<(String, f32), bliss_audio::BlissError>>>,
    errors: Vec<bliss_audio::BlissError>,
}

#[cfg(feature = "bliss-audio")]
impl BlissSorter {
    fn get_or_wait(&mut self, path: &str) -> Option<f32> {
        if let Some(distance) = self.float_map.get(path) {
            Some(*distance)
        } else {
            // wait on threads until matching result is found
            for result in self.rx.as_ref().unwrap() {
                match result {
                    Ok((key, distance)) => {
                        if path == key {
                            self.float_map.insert(key, distance);
                            return Some(distance);
                        } else {
                            self.float_map.insert(key, distance);
                        }
                    }
                    Err(e) => {
                        self.errors.push(e);
                        return None;
                    }
                }
            }
            None
        }
    }

    #[inline]
    fn compare_songs(
        song1: Song,
        path_2: String,
    ) -> Result<(String, f32), bliss_audio::BlissError> {
        let song2 = Song::new(&path_2)?;
        let distance = song1.distance(&song2);
        Ok((path_2, distance))
    }
}

#[cfg(feature = "bliss-audio")]
impl std::clone::Clone for BlissSorter {
    fn clone(&self) -> Self {
        Self {
            up_to: self.up_to,
            float_map: self.float_map.clone(),
            first_song: self.first_song.clone(),
            rx: None,
            errors: Vec::new(),
        }
    }
}

#[cfg(feature = "bliss-audio")]
impl Default for BlissSorter {
    fn default() -> Self {
        Self {
            up_to: usize::MAX,
            float_map: HashMap::new(),
            first_song: None,
            rx: None,
            errors: Vec::new(),
        }
    }
}

#[cfg(feature = "bliss-audio")]
impl MpsSorter for BlissSorter {
    fn sort(
        &mut self,
        iterator: &mut dyn MpsOp,
        item_buf: &mut VecDeque<MpsIteratorItem>,
        op: &mut OpGetter,
    ) -> Result<(), RuntimeError> {
        let buf_len_old = item_buf.len(); // save buffer length before modifying buffer
        if item_buf.len() < self.up_to {
            for item in iterator {
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
                let (tx_chann, rx_chann) = channel();
                let mut item_paths = Vec::with_capacity(item_buf.len() - 1);
                for item in item_buf.iter() {
                    if let Ok(item) = item {
                        // build comparison table
                        if let Some(MpsTypePrimitive::String(path)) = item.field("filename") {
                            if self.first_song.is_none() {
                                // find first valid song (ie first item with field "filename")
                                self.first_song = Some(path.to_owned());
                                //self.first_song = Some(Song::new(path).map_err(|e| bliss_err(e, op))?);
                                self.float_map.insert(path.to_owned(), 0.0); // distance to itself should be 0
                            } else {
                                item_paths.push(path.to_owned());
                            }
                        }
                    }
                }
                if let Some(first_song_path) = &self.first_song {
                    // spawn threads for processing song distances
                    let path1_clone = first_song_path.to_owned();
                    std::thread::spawn(move || match Song::new(path1_clone) {
                        Err(e) => tx_chann.send(Err(e)).unwrap_or(()),
                        Ok(song1) => {
                            for path2 in item_paths {
                                let result_chann = tx_chann.clone();
                                let song1_clone = song1.clone();
                                std::thread::spawn(move || {
                                    result_chann
                                        .send(Self::compare_songs(song1_clone, path2))
                                        .unwrap_or(());
                                });
                            }
                        }
                    });
                }
                self.rx = Some(rx_chann);
                // unordered list returned on first call to this function
                // note that only the first item will be used by sorter,
                // since the second time this function is called the remaining items are sorted properly
            }
        } else if self.first_song.is_some() {
            // Sort songs on second call to this function
            self.first_song = None;
            item_buf.make_contiguous().sort_by(|a, b| {
                if let Ok(a) = a {
                    if let Some(MpsTypePrimitive::String(a_path)) = a.field("filename") {
                        if let Ok(b) = b {
                            if let Some(MpsTypePrimitive::String(b_path)) = b.field("filename") {
                                if let Some(float_a) = self.get_or_wait(a_path) {
                                    if let Some(float_b) = self.get_or_wait(b_path) {
                                        return float_a
                                            .partial_cmp(&float_b)
                                            .unwrap_or(DEFAULT_ORDER);
                                    }
                                }
                            }
                        }
                    }
                }
                DEFAULT_ORDER
            });
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(bliss_err(self.errors.pop().unwrap(), op))
        }
    }
}

#[cfg(feature = "bliss-audio")]
fn bliss_err<D: Display>(error: D, op: &mut OpGetter) -> RuntimeError {
    RuntimeError {
        line: 0,
        op: op(),
        msg: format!("Bliss error: {}", error),
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
