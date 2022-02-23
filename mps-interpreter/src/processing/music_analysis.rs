use core::fmt::Debug;
#[cfg(feature = "bliss-audio")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "bliss-audio")]
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(feature = "bliss-audio")]
use crate::lang::MpsTypePrimitive;
#[cfg(feature = "bliss-audio")]
use bliss_audio::{BlissError, Song};

use crate::lang::RuntimeMsg;
use crate::MpsItem;

const PATH_FIELD: &str = "filename";

pub trait MpsMusicAnalyzer: Debug {
    fn prepare_distance(&mut self, from: &MpsItem, to: &MpsItem) -> Result<(), RuntimeMsg>;

    fn prepare_item(&mut self, item: &MpsItem) -> Result<(), RuntimeMsg>;

    fn get_distance(&mut self, from: &MpsItem, to: &MpsItem) -> Result<f64, RuntimeMsg>;

    fn clear_cache(&mut self) -> Result<(), RuntimeMsg>;
}

#[cfg(feature = "bliss-audio")]
#[derive(Debug)]
pub struct MpsDefaultAnalyzer {
    requests: Sender<RequestType>,
    responses: Receiver<ResponseType>,
}

#[cfg(feature = "bliss-audio")]
impl std::default::Default for MpsDefaultAnalyzer {
    fn default() -> Self {
        let (req_tx, req_rx) = channel();
        let (resp_tx, resp_rx) = channel();
        std::thread::spawn(move || {
            let mut cache_thread = CacheThread::new(resp_tx);
            cache_thread.run_loop(req_rx);
        });
        Self {
            requests: req_tx,
            responses: resp_rx,
        }
    }
}

#[cfg(feature = "bliss-audio")]
impl MpsDefaultAnalyzer {
    fn request_distance(
        &mut self,
        from: &MpsItem,
        to: &MpsItem,
        ack: bool,
    ) -> Result<(), RuntimeMsg> {
        let path_from = Self::get_path(from)?;
        let path_to = Self::get_path(to)?;
        self.requests
            .send(RequestType::Distance {
                path1: path_from.to_owned(),
                path2: path_to.to_owned(),
                ack: ack,
            })
            .map_err(|e| RuntimeMsg(format!("Channel send err {}", e)))
    }

    fn get_path(item: &MpsItem) -> Result<&str, RuntimeMsg> {
        if let Some(path) = item.field(PATH_FIELD) {
            if let MpsTypePrimitive::String(path) = path {
                Ok(path)
            } else {
                Err(RuntimeMsg(format!(
                    "Field {} on item is not String, it's {}",
                    PATH_FIELD, path
                )))
            }
        } else {
            Err(RuntimeMsg(format!("Missing field {} on item", PATH_FIELD)))
        }
    }

    fn request_song(&mut self, item: &MpsItem, ack: bool) -> Result<(), RuntimeMsg> {
        let path = Self::get_path(item)?;
        self.requests
            .send(RequestType::Song {
                path: path.to_owned(),
                ack: ack,
            })
            .map_err(|e| RuntimeMsg(format!("Channel send error: {}", e)))
    }
}

#[cfg(feature = "bliss-audio")]
impl MpsMusicAnalyzer for MpsDefaultAnalyzer {
    fn prepare_distance(&mut self, from: &MpsItem, to: &MpsItem) -> Result<(), RuntimeMsg> {
        self.request_distance(from, to, false)
    }

    fn prepare_item(&mut self, item: &MpsItem) -> Result<(), RuntimeMsg> {
        self.request_song(item, false)
    }

    fn get_distance(&mut self, from: &MpsItem, to: &MpsItem) -> Result<f64, RuntimeMsg> {
        self.request_distance(from, to, true)?;
        let path_from = Self::get_path(from)?;
        let path_to = Self::get_path(to)?;
        for response in self.responses.iter() {
            if let ResponseType::Distance {
                path1,
                path2,
                distance,
            } = response
            {
                if path1 == path_from && path2 == path_to {
                    return match distance {
                        Ok(d) => Ok(d as f64),
                        Err(e) => Err(RuntimeMsg(format!("Bliss error: {}", e))),
                    };
                }
            }
        }
        Err(RuntimeMsg(
            "Channel closed without response: internal error".to_owned(),
        ))
    }

    fn clear_cache(&mut self) -> Result<(), RuntimeMsg> {
        self.requests
            .send(RequestType::Clear {})
            .map_err(|e| RuntimeMsg(format!("Channel send error: {}", e)))
    }
}

#[cfg(not(feature = "bliss-audio"))]
#[derive(Default, Debug)]
pub struct MpsDefaultAnalyzer {}

#[cfg(not(feature = "bliss-audio"))]
impl MpsMusicAnalyzer for MpsDefaultAnalyzer {
    fn prepare_distance(&mut self, from: &MpsItem, to: &MpsItem) -> Result<(), RuntimeMsg> {
        Ok(())
    }

    fn prepare_item(&mut self, item: &MpsItem) -> Result<(), RuntimeMsg> {
        Ok(())
    }

    fn get_distance(&mut self, item: &MpsItem) -> Result<f64, RuntimeMsg> {
        Ok(f64::MAX)
    }
}

#[cfg(feature = "bliss-audio")]
enum RequestType {
    Distance {
        path1: String,
        path2: String,
        ack: bool,
    },
    Song {
        path: String,
        ack: bool,
    },
    Clear {},
    //End {}
}

#[cfg(feature = "bliss-audio")]
enum ResponseType {
    Distance {
        path1: String,
        path2: String,
        distance: Result<f32, BlissError>,
    },
    Song {
        path: String,
        song: Result<Song, BlissError>,
    },
}

#[cfg(feature = "bliss-audio")]
struct CacheThread {
    distance_cache: HashMap<(String, String), Result<f32, BlissError>>,
    distance_in_progress: HashSet<(String, String)>,
    song_cache: HashMap<String, Result<Song, BlissError>>,
    song_in_progress: HashSet<String>,
    //requests: Receiver<RequestType>,
    responses: Sender<ResponseType>,
}

#[cfg(feature = "bliss-audio")]
impl CacheThread {
    fn new(responses: Sender<ResponseType>) -> Self {
        Self {
            distance_cache: HashMap::new(),
            distance_in_progress: HashSet::new(),
            song_cache: HashMap::new(),
            song_in_progress: HashSet::new(),
            //requests: requests,
            responses: responses,
        }
    }

    fn non_blocking_read_some(&mut self, results: &Receiver<ResponseType>) {
        for result in results.try_iter() {
            match result {
                ResponseType::Distance {
                    path1,
                    path2,
                    distance,
                } => {
                    self.insert_distance(path1, path2, distance);
                }
                ResponseType::Song { path, song } => {
                    self.insert_song(path, song);
                }
            }
        }
    }

    fn insert_song(&mut self, path: String, song_result: Result<Song, BlissError>) {
        self.song_in_progress.remove(&path);
        self.song_cache.insert(path, song_result);
    }

    fn insert_distance(
        &mut self,
        path1: String,
        path2: String,
        distance_result: Result<f32, BlissError>,
    ) {
        let key = (path1, path2);
        self.distance_in_progress.remove(&key);
        self.distance_cache.insert(key, distance_result);
    }

    fn get_song_option(
        &mut self,
        path: &str,
        auto_add: bool,
        results: &Receiver<ResponseType>,
    ) -> Option<Song> {
        // wait for song if already in progress
        if self.song_in_progress.contains(path) {
            for result in results.iter() {
                match result {
                    ResponseType::Distance {
                        path1,
                        path2,
                        distance,
                    } => {
                        self.insert_distance(path1, path2, distance);
                    }
                    ResponseType::Song { path: path2, song } => {
                        if path2 == path {
                            self.insert_song(path2, song.clone());
                            let result = song.ok();
                            if result.is_none() && auto_add {
                                self.song_in_progress.insert(path.to_owned());
                            }
                            return result;
                        } else {
                            self.insert_song(path2, song);
                        }
                    }
                }
            }
        } else if self.song_cache.contains_key(path) {
            let result = self
                .song_cache
                .get(path)
                .and_then(|r| r.clone().ok().to_owned());
            if result.is_none() && auto_add {
                self.song_in_progress.insert(path.to_owned());
            }
            return result;
        }
        if auto_add {
            self.song_in_progress.insert(path.to_owned());
        }
        return None;
    }

    fn handle_distance_req(
        &mut self,
        path1: String,
        path2: String,
        ack: bool,
        worker_tx: &Sender<ResponseType>,
        worker_results: &Receiver<ResponseType>,
    ) -> bool {
        let key = (path1.clone(), path2.clone());
        if let Some(result) = self.distance_cache.get(&key) {
            if ack {
                let result = result.to_owned();
                if let Err(_) = self.responses.send(ResponseType::Distance {
                    path1: path1,
                    path2: path2,
                    distance: result,
                }) {
                    return true;
                }
            }
        } else {
            if path1 == path2 {
                // trivial case
                // also prevents deadlock in self.get_song_option()
                // due to waiting on song that isn't being processed yet
                // (first call adds it to song_in_progress set, second call just waits)
                if ack {
                    if let Err(_) = self.responses.send(ResponseType::Distance {
                        path1: path1,
                        path2: path2,
                        distance: Ok(0.0),
                    }) {
                        return true;
                    }
                }
            } else if !self.distance_in_progress.contains(&key) {
                let results = worker_tx.clone();
                let song1_clone = self.get_song_option(&path1, true, worker_results);
                let song2_clone = self.get_song_option(&path2, true, worker_results);
                std::thread::spawn(move || {
                    let distance_result =
                        worker_distance(&results, (&path1, song1_clone), (&path2, song2_clone));
                    results
                        .send(ResponseType::Distance {
                            path1: path1,
                            path2: path2,
                            distance: distance_result,
                        })
                        .unwrap_or(());
                });
            }
            if ack {
                'inner1: for result in worker_results.iter() {
                    match result {
                        ResponseType::Distance {
                            path1: path1_2,
                            path2: path2_2,
                            distance,
                        } => {
                            self.insert_distance(
                                path1_2.clone(),
                                path2_2.clone(),
                                distance.clone(),
                            );
                            if path1_2 == key.0 && path2_2 == key.1 {
                                if let Err(_) = self.responses.send(ResponseType::Distance {
                                    path1: path1_2,
                                    path2: path2_2,
                                    distance: distance,
                                }) {
                                    return true;
                                }
                                break 'inner1;
                            }
                        }
                        ResponseType::Song { path, song } => {
                            self.insert_song(path, song);
                        }
                    }
                }
            }
        }
        false
    }

    fn handle_song_req(
        &mut self,
        path: String,
        ack: bool,
        worker_tx: &Sender<ResponseType>,
        worker_results: &Receiver<ResponseType>,
    ) -> bool {
        if let Some(song) = self.song_cache.get(&path) {
            if ack {
                let song = song.to_owned();
                if let Err(_) = self.responses.send(ResponseType::Song {
                    path: path,
                    song: song,
                }) {
                    return true;
                }
            }
        } else {
            if !self.song_in_progress.contains(&path) {
                let path_clone = path.clone();
                let results = worker_tx.clone();
                std::thread::spawn(move || {
                    let song_result = Song::new(&path_clone);
                    results
                        .send(ResponseType::Song {
                            path: path_clone,
                            song: song_result,
                        })
                        .unwrap_or(());
                });
            }
            if ack {
                'inner2: for result in worker_results.iter() {
                    match result {
                        ResponseType::Distance {
                            path1,
                            path2,
                            distance,
                        } => {
                            self.insert_distance(path1, path2, distance);
                        }
                        ResponseType::Song { path: path2, song } => {
                            self.insert_song(path2.clone(), song.clone());
                            if path2 == path {
                                if let Err(_) = self.responses.send(ResponseType::Song {
                                    path: path,
                                    song: song,
                                }) {
                                    return false;
                                }
                                break 'inner2;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn run_loop(&mut self, requests: Receiver<RequestType>) {
        let (worker_tx, worker_results): (Sender<ResponseType>, Receiver<ResponseType>) = channel();
        'outer: for request in requests.iter() {
            self.non_blocking_read_some(&worker_results);
            match request {
                //RequestType::End{} => break,
                RequestType::Distance { path1, path2, ack } => {
                    if self.handle_distance_req(path1, path2, ack, &worker_tx, &worker_results) {
                        break 'outer;
                    }
                }
                RequestType::Song { path, ack } => {
                    if self.handle_song_req(path, ack, &worker_tx, &worker_results) {
                        break 'outer;
                    }
                }
                RequestType::Clear {} => {
                    self.distance_cache.clear();
                    self.song_cache.clear();
                }
            }
        }
    }
}

#[cfg(feature = "bliss-audio")]
fn worker_distance(
    results: &Sender<ResponseType>,
    song1: (&str, Option<Song>),
    song2: (&str, Option<Song>),
) -> Result<f32, BlissError> {
    let path1 = song1.0;
    let song1 = if let Some(song) = song1.1 {
        song
    } else {
        let new_song1 = Song::new(path1);
        results
            .send(ResponseType::Song {
                path: path1.to_string(),
                song: new_song1.clone(),
            })
            .unwrap_or(());
        new_song1?
    };
    let path2 = song2.0;
    let song2 = if let Some(song) = song2.1 {
        song
    } else {
        let new_song2 = Song::new(path2);
        results
            .send(ResponseType::Song {
                path: path2.to_string(),
                song: new_song2.clone(),
            })
            .unwrap_or(());
        new_song2?
    };
    Ok(song1.distance(&song2))
}
