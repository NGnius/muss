use std::collections::VecDeque;
#[cfg(feature = "bliss-audio")]
use std::fmt::{Debug, Display, Error, Formatter};
#[cfg(feature = "bliss-audio")]
use std::sync::mpsc::{channel, Receiver, Sender};

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
use crate::MpsItem;

#[cfg(feature = "bliss-audio")]
#[derive(Debug)]
pub struct BlissNextSorter {
    up_to: usize,
    rx: Option<Receiver<Option<Result<MpsItem, bliss_audio::BlissError>>>>,
    algorithm_done: bool,
}

#[cfg(feature = "bliss-audio")]
impl BlissNextSorter {
    fn get_maybe(&mut self, op: &mut OpGetter) -> Option<MpsIteratorItem> {
        if self.algorithm_done {
            None
        } else if let Ok(Some(item)) = self.rx.as_ref().unwrap().recv() {
            Some(item.map_err(|e| bliss_err(e, op)))
        } else {
            self.algorithm_done = true;
            None
        }
    }

    fn algorithm(
        mut items: VecDeque<MpsItem>,
        results: Sender<Option<Result<MpsItem, bliss_audio::BlissError>>>,
    ) {
        let mut song_cache: Option<(Song, String)> = None;
        let items_len = items.len();
        for i in 0..items_len {
            let item = items.pop_front().unwrap();
            if let Some(MpsTypePrimitive::String(path)) = item.field("filename") {
                if let Err(_) = results.send(Some(Ok(item.clone()))) {
                    break;
                }
                if i + 2 < items_len {
                    let target_song = if let Some((_, ref cached_filename)) = song_cache {
                        if cached_filename == path {
                            Ok(song_cache.take().unwrap().0)
                        } else {
                            Song::new(path)
                        }
                    } else {
                        Song::new(path)
                    };
                    let target_song = match target_song {
                        Ok(x) => x,
                        Err(e) => {
                            results.send(Some(Err(e))).unwrap_or(());
                            break;
                        }
                    };
                    match Self::find_best(&items, target_song) {
                        Err(e) => {
                            results.send(Some(Err(e))).unwrap_or(());
                            break;
                        }
                        Ok((next_song, index)) => {
                            if let Some(next_song) = next_song {
                                if index != 0 {
                                    items.swap(0, index);
                                }
                                song_cache = Some((next_song, path.to_owned()));
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
        results.send(None).unwrap_or(());
    }

    fn find_best(
        items: &VecDeque<MpsItem>,
        target: Song,
    ) -> Result<(Option<Song>, usize), bliss_audio::BlissError> {
        let mut best = None;
        let mut best_index = 0;
        let mut best_distance = f32::MAX;
        let (tx, rx) = channel();
        let mut threads_spawned = 0;
        for i in 0..items.len() {
            if let Some(MpsTypePrimitive::String(path)) = items[i].field("filename") {
                let result_chann = tx.clone();
                let target_clone = target.clone();
                let path_clone = path.to_owned();
                std::thread::spawn(move || match Song::new(path_clone) {
                    Err(e) => result_chann.send(Err(e)).unwrap_or(()),
                    Ok(song) => result_chann
                        .send(Ok((i, target_clone.distance(&song), song)))
                        .unwrap_or(()),
                });
                threads_spawned += 1;
            }
        }
        for _ in 0..threads_spawned {
            if let Ok(result) = rx.recv() {
                let (index, distance, song) = result?;
                if distance < best_distance {
                    best = Some(song);
                    best_index = index;
                    best_distance = distance;
                }
            } else {
                break;
            }
        }
        Ok((best, best_index))
    }
}

#[cfg(feature = "bliss-audio")]
impl std::clone::Clone for BlissNextSorter {
    fn clone(&self) -> Self {
        Self {
            up_to: self.up_to,
            rx: None,
            algorithm_done: self.algorithm_done,
        }
    }
}

#[cfg(feature = "bliss-audio")]
impl Default for BlissNextSorter {
    fn default() -> Self {
        Self {
            up_to: usize::MAX,
            rx: None,
            algorithm_done: false,
        }
    }
}

#[cfg(feature = "bliss-audio")]
impl MpsSorter for BlissNextSorter {
    fn sort(
        &mut self,
        iterator: &mut dyn MpsOp,
        item_buf: &mut VecDeque<MpsIteratorItem>,
        op: &mut OpGetter,
    ) -> Result<(), RuntimeError> {
        if self.rx.is_none() {
            // first run
            let mut items = VecDeque::new();
            for item in iterator {
                match item {
                    Ok(item) => items.push_back(item),
                    Err(e) => item_buf.push_back(Err(e)),
                }
                if items.len() + item_buf.len() >= self.up_to {
                    break;
                }
            }
            // start algorithm
            let (tx, rx) = channel();
            std::thread::spawn(move || Self::algorithm(items, tx));
            self.rx = Some(rx);
        }
        if let Some(item) = self.get_maybe(op) {
            item_buf.push_back(item);
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.algorithm_done = false;
        self.rx = None;
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
pub type BlissNextSorter = crate::lang::vocabulary::sorters::EmptySorter;

#[cfg(feature = "bliss-audio")]
impl Display for BlissNextSorter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "advanced bliss_next")
    }
}

pub struct BlissNextSorterFactory;

impl MpsSorterFactory<BlissNextSorter> for BlissNextSorterFactory {
    fn is_sorter(&self, tokens: &VecDeque<&MpsToken>) -> bool {
        tokens.len() == 2
            && check_name("advanced", tokens[0])
            && check_name("bliss_next", tokens[1])
    }

    fn build_sorter(
        &self,
        tokens: &mut VecDeque<MpsToken>,
        _dict: &MpsLanguageDictionary,
    ) -> Result<BlissNextSorter, SyntaxError> {
        assert_name("advanced", tokens)?;
        assert_name("bliss_next", tokens)?;
        Ok(BlissNextSorter::default())
    }
}

pub type BlissNextSorterStatementFactory =
    MpsSortStatementFactory<BlissNextSorter, BlissNextSorterFactory>;

#[inline(always)]
pub fn bliss_next_sort() -> BlissNextSorterStatementFactory {
    BlissNextSorterStatementFactory::new(BlissNextSorterFactory)
}
