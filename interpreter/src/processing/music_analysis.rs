use core::fmt::Debug;
#[cfg(feature = "bliss-audio-symphonia")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "bliss-audio-symphonia")]
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(feature = "bliss-audio-symphonia")]
use crate::lang::TypePrimitive;
#[cfg(feature = "bliss-audio-symphonia")]
use bliss_audio_symphonia::{BlissError, Song, AnalysisIndex};

// assumed processor threads
const DEFAULT_PARALLELISM: usize = 2;

// maximum length of song cache (song objects take up a lot of memory)
const MAX_SONG_CACHE_SIZE: usize = 10000;

// maximum length of distance cache (takes up significantly less memory than songs)
const MAX_DISTANCE_CACHE_SIZE: usize = MAX_SONG_CACHE_SIZE * MAX_SONG_CACHE_SIZE;

use crate::lang::RuntimeMsg;
use crate::Item;

const PATH_FIELD: &str = "filename";

#[derive(Debug, Clone)]
pub enum MusicAnalyzerDistance {
    Tempo,
    Spectrum,
    Loudness,
    Chroma,
}

pub trait MusicAnalyzer: Debug + Send {
    fn prepare_distance(&mut self, from: &Item, to: &Item) -> Result<(), RuntimeMsg>;

    fn prepare_item(&mut self, item: &Item) -> Result<(), RuntimeMsg>;

    fn get_distance(&mut self, from: &Item, to: &Item) -> Result<f64, RuntimeMsg>;

    fn get_custom_distance(&mut self, from: &Item, to: &Item, compare: MusicAnalyzerDistance) -> Result<f64, RuntimeMsg>;

    fn clear_cache(&mut self) -> Result<(), RuntimeMsg>;
}

#[cfg(feature = "bliss-audio-symphonia")]
#[derive(Debug)]
pub struct DefaultAnalyzer {
    requests: Sender<RequestType>,
    responses: Receiver<ResponseType>,
}

#[cfg(feature = "bliss-audio-symphonia")]
impl std::default::Default for DefaultAnalyzer {
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

#[cfg(feature = "bliss-audio-symphonia")]
impl DefaultAnalyzer {
    fn request_distance(&mut self, from: &Item, to: &Item, ack: bool) -> Result<(), RuntimeMsg> {
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

    fn get_path(item: &Item) -> Result<&str, RuntimeMsg> {
        if let Some(path) = item.field(PATH_FIELD) {
            if let TypePrimitive::String(path) = path {
                if path.starts_with("file://") {
                    //println!("path guess: `{}`", path.get(7..).unwrap());
                    Ok(path.get(7..).unwrap())
                } else if !path.contains("://") {
                    Ok(path)
                } else {
                    Err(RuntimeMsg(format!(
                        "Field {} on item is not a supported URI, it's {}",
                        PATH_FIELD, path
                    )))
                }
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

    fn request_song(&mut self, item: &Item, ack: bool) -> Result<(), RuntimeMsg> {
        let path = Self::get_path(item)?;
        self.requests
            .send(RequestType::Song {
                path: path.to_owned(),
                ack: ack,
            })
            .map_err(|e| RuntimeMsg(format!("Channel send error: {}", e)))
    }

    fn bliss_song_to_array(song: &Song) -> [f64; bliss_audio_symphonia::NUMBER_FEATURES] {
        let analysis = &song.analysis;
        [
            analysis[AnalysisIndex::Tempo] as _,
            analysis[AnalysisIndex::Zcr] as _,
            analysis[AnalysisIndex::MeanSpectralCentroid] as _,
            analysis[AnalysisIndex::StdDeviationSpectralCentroid] as _,
            analysis[AnalysisIndex::MeanSpectralRolloff] as _,
            analysis[AnalysisIndex::StdDeviationSpectralRolloff] as _,
            analysis[AnalysisIndex::MeanSpectralFlatness] as _,
            analysis[AnalysisIndex::StdDeviationSpectralFlatness] as _,
            analysis[AnalysisIndex::MeanLoudness] as _,
            analysis[AnalysisIndex::StdDeviationLoudness] as _,
            analysis[AnalysisIndex::Chroma1] as _,
            analysis[AnalysisIndex::Chroma2] as _,
            analysis[AnalysisIndex::Chroma3] as _,
            analysis[AnalysisIndex::Chroma4] as _,
            analysis[AnalysisIndex::Chroma5] as _,
            analysis[AnalysisIndex::Chroma6] as _,
            analysis[AnalysisIndex::Chroma7] as _,
            analysis[AnalysisIndex::Chroma8] as _,
            analysis[AnalysisIndex::Chroma9] as _,
            analysis[AnalysisIndex::Chroma10] as _,
        ]
    }
}

#[cfg(feature = "bliss-audio-symphonia")]
impl MusicAnalyzer for DefaultAnalyzer {
    fn prepare_distance(&mut self, from: &Item, to: &Item) -> Result<(), RuntimeMsg> {
        self.request_distance(from, to, false)
    }

    fn prepare_item(&mut self, item: &Item) -> Result<(), RuntimeMsg> {
        self.request_song(item, false)
    }

    fn get_distance(&mut self, from: &Item, to: &Item) -> Result<f64, RuntimeMsg> {
        self.request_distance(from, to, true)?;
        let path_from = Self::get_path(from)?;
        let path_to = Self::get_path(to)?;
        for response in self.responses.iter() {
            match response {
                ResponseType::Distance {
                    path1,
                    path2,
                    distance,
                } => {
                    //println!("Got distance from `{}` to `{}`: {}", path1, path2, distance.as_ref().ok().unwrap_or(&f32::INFINITY));
                    if path1 == path_from && path2 == path_to {
                        return match distance {
                            Ok(d) => Ok(d as f64),
                            Err(e) => Err(RuntimeMsg(format!("Bliss error: {}", e))),
                        };
                    }
                }
                ResponseType::Song { .. } => {}
                ResponseType::UnsupportedSong { path, msg } => {
                    if path == path_to || path == path_from {
                        return Err(RuntimeMsg(format!("Bliss error: {}", msg)));
                    }
                }
            }
        }
        Err(RuntimeMsg(
            "Channel closed without response: internal error".to_owned(),
        ))
    }

    fn get_custom_distance(&mut self, from: &Item, to: &Item, compare: MusicAnalyzerDistance) -> Result<f64, RuntimeMsg> {
        self.request_song(from, true)?;
        self.request_song(to, true)?;
        let path_from = Self::get_path(from)?;
        let path_to = Self::get_path(to)?;
        let mut from_song = None;
        let mut to_song = None;
        for response in self.responses.iter() {
            match response {
                ResponseType::Distance { .. } => {},
                ResponseType::Song {
                    path,
                    song
                } => {
                    if path_from == path {
                        from_song = Some(song.map_err(|e| RuntimeMsg(format!("Bliss error: {}", e)))?);
                    } else if path_to == path {
                        to_song = Some(song.map_err(|e| RuntimeMsg(format!("Bliss error: {}", e)))?);
                    }
                    if to_song.is_some() && from_song.is_some() {
                        break;
                    }
                },
                ResponseType::UnsupportedSong { path, msg } => {
                    if path == path_to || path == path_from {
                        return Err(RuntimeMsg(format!("Bliss error: {}", msg)));
                    }
                }
            }
        }
        if to_song.is_some() && from_song.is_some() {
            let to_arr = Self::bliss_song_to_array(&to_song.unwrap());
            let from_arr = Self::bliss_song_to_array(&from_song.unwrap());
            Ok(match compare {
                MusicAnalyzerDistance::Tempo => (
                    (to_arr[AnalysisIndex::Tempo as usize] - from_arr[AnalysisIndex::Tempo as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Zcr as usize] - from_arr[AnalysisIndex::Zcr as usize]).powi(2)
                ).sqrt(),
                MusicAnalyzerDistance::Spectrum => (
                    (to_arr[AnalysisIndex::MeanSpectralCentroid as usize] - from_arr[AnalysisIndex::MeanSpectralCentroid as usize]).powi(2)
                    + (to_arr[AnalysisIndex::StdDeviationSpectralCentroid as usize] - from_arr[AnalysisIndex::StdDeviationSpectralCentroid as usize]).powi(2)
                    + (to_arr[AnalysisIndex::MeanSpectralRolloff as usize] - from_arr[AnalysisIndex::MeanSpectralRolloff as usize]).powi(2)
                    + (to_arr[AnalysisIndex::StdDeviationSpectralRolloff as usize] - from_arr[AnalysisIndex::StdDeviationSpectralRolloff as usize]).powi(2)
                    + (to_arr[AnalysisIndex::MeanSpectralFlatness as usize] - from_arr[AnalysisIndex::MeanSpectralFlatness as usize]).powi(2)
                    + (to_arr[AnalysisIndex::StdDeviationSpectralFlatness as usize] - from_arr[AnalysisIndex::StdDeviationSpectralFlatness as usize]).powi(2)
                ).sqrt(),
                MusicAnalyzerDistance::Loudness => {
                    let mean_delta = to_arr[AnalysisIndex::MeanLoudness as usize] - from_arr[AnalysisIndex::MeanLoudness as usize];
                    let deviation_delta = to_arr[AnalysisIndex::StdDeviationLoudness as usize] - from_arr[AnalysisIndex::StdDeviationLoudness as usize];

                    (mean_delta.powi(2) + deviation_delta.powi(2)).sqrt()
                },
                MusicAnalyzerDistance::Chroma => (
                    (to_arr[AnalysisIndex::Chroma1 as usize] - from_arr[AnalysisIndex::Chroma1 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma2 as usize] - from_arr[AnalysisIndex::Chroma2 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma3 as usize] - from_arr[AnalysisIndex::Chroma3 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma4 as usize] - from_arr[AnalysisIndex::Chroma4 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma5 as usize] - from_arr[AnalysisIndex::Chroma5 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma6 as usize] - from_arr[AnalysisIndex::Chroma6 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma7 as usize] - from_arr[AnalysisIndex::Chroma7 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma8 as usize] - from_arr[AnalysisIndex::Chroma8 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma9 as usize] - from_arr[AnalysisIndex::Chroma9 as usize]).powi(2)
                    + (to_arr[AnalysisIndex::Chroma10 as usize] - from_arr[AnalysisIndex::Chroma10 as usize]).powi(2)
                ).sqrt(),
            })
        } else {
            Err(RuntimeMsg(
                "Channel closed without complete response: internal error".to_owned(),
            ))
        }
    }

    fn clear_cache(&mut self) -> Result<(), RuntimeMsg> {
        self.requests
            .send(RequestType::Clear {})
            .map_err(|e| RuntimeMsg(format!("Channel send error: {}", e)))
    }
}

#[cfg(not(feature = "bliss-audio-symphonia"))]
#[derive(Default, Debug)]
pub struct DefaultAnalyzer {}

#[cfg(not(feature = "bliss-audio-symphonia"))]
impl MusicAnalyzer for DefaultAnalyzer {
    fn prepare_distance(&mut self, _from: &Item, _to: &Item) -> Result<(), RuntimeMsg> {
        Ok(())
    }

    fn prepare_item(&mut self, _item: &Item) -> Result<(), RuntimeMsg> {
        Ok(())
    }

    fn get_distance(&mut self, _from: &Item, _to: &Item) -> Result<f64, RuntimeMsg> {
        Ok(f64::MAX)
    }

    fn get_custom_distance(&mut self, _from: &Item, _to: &Item, _compare: MusicAnalyzerDistance) -> Result<f64, RuntimeMsg> {
        Ok(f64::MAX)
    }

    fn clear_cache(&mut self) -> Result<(), RuntimeMsg> {
        Ok(())
    }
}

#[cfg(feature = "bliss-audio-symphonia")]
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

#[cfg(feature = "bliss-audio-symphonia")]
enum ResponseType {
    Distance {
        path1: String,
        path2: String,
        distance: Result<f32, BlissError>,
    },
    Song {
        path: String,
        song: Result<Box<Song>, BlissError>,
    },
    UnsupportedSong {
        path: String,
        msg: String,
    },
}

#[cfg(feature = "bliss-audio-symphonia")]
struct CacheThread {
    distance_cache: HashMap<(String, String), Result<f32, BlissError>>,
    distance_in_progress: HashSet<(String, String)>,
    song_cache: HashMap<String, Result<Song, BlissError>>,
    song_in_progress: HashSet<String>,
    //requests: Receiver<RequestType>,
    responses: Sender<ResponseType>,
}

#[cfg(feature = "bliss-audio-symphonia")]
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
                ResponseType::UnsupportedSong { .. } => {}
            }
        }
    }

    fn insert_song(&mut self, path: String, song_result: Result<Box<Song>, BlissError>) {
        self.song_in_progress.remove(&path);
        if self.song_cache.len() > MAX_SONG_CACHE_SIZE {
            // avoid using too much memory -- songs are big memory objects
            self.song_cache.clear();
        }
        self.song_cache
            .insert(path, song_result.map(|x| x.as_ref().to_owned()));
    }

    fn insert_distance(
        &mut self,
        path1: String,
        path2: String,
        distance_result: Result<f32, BlissError>,
    ) {
        let key = (path1, path2);
        self.distance_in_progress.remove(&key);
        if self.distance_cache.len() > MAX_DISTANCE_CACHE_SIZE {
            // avoid using too much memory
            self.distance_cache.clear();
        }
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
                            return result.map(|x| x.as_ref().to_owned());
                        } else {
                            self.insert_song(path2, song);
                        }
                    }
                    ResponseType::UnsupportedSong {
                        path: unsupported_path,
                        ..
                    } => {
                        self.song_in_progress.remove(&unsupported_path);
                        if path == unsupported_path {
                            return None;
                        }
                    }
                }
            }
        } else if self.song_cache.contains_key(path) {
            let result = self.song_cache.get(path).and_then(|r| r.clone().ok());
            if result.is_none() && auto_add {
                self.song_in_progress.insert(path.to_owned());
            }
            return result;
        }
        if auto_add {
            self.song_in_progress.insert(path.to_owned());
        }
        None
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
                if self
                    .responses
                    .send(ResponseType::Distance {
                        path1: path1,
                        path2: path2,
                        distance: result,
                    })
                    .is_err()
                {
                    return true;
                }
            }
        } else {
            if path1 == path2 {
                // trivial case
                // also prevents deadlock in self.get_song_option()
                // due to waiting on song that isn't being processed yet
                // (first call adds it to song_in_progress set, second call just waits)
                if ack
                    && self
                        .responses
                        .send(ResponseType::Distance {
                            path1: path1,
                            path2: path2,
                            distance: Ok(0.0),
                        })
                        .is_err()
                {
                    return true;
                }
            } else if !self.distance_in_progress.contains(&key) {
                // distance worker uses 3 threads (it's own thread + 1 extra per song) for 2 songs
                let available_parallelism = (std::thread::available_parallelism()
                    .ok()
                    .map(|x| x.get())
                    .unwrap_or(DEFAULT_PARALLELISM)
                    * 2)
                    / 3;
                let available_parallelism = if available_parallelism != 0 {
                    available_parallelism - 1
                } else {
                    0
                };
                // wait for processing to complete if too many tasks already running
                if self.song_in_progress.len() > available_parallelism {
                    'inner4: for result in worker_results.iter() {
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
                                if self.song_in_progress.len() <= available_parallelism {
                                    break 'inner4;
                                }
                            }
                            ResponseType::UnsupportedSong {
                                path: unsupported_path,
                                ..
                            } => {
                                self.song_in_progress.remove(&unsupported_path);
                                if self.song_in_progress.len() <= available_parallelism {
                                    break 'inner4;
                                }
                            }
                        }
                    }
                }
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
                                if self
                                    .responses
                                    .send(ResponseType::Distance {
                                        path1: path1_2,
                                        path2: path2_2,
                                        distance: distance,
                                    })
                                    .is_err()
                                {
                                    return true;
                                }
                                break 'inner1;
                            }
                        }
                        ResponseType::Song { path, song } => {
                            self.insert_song(path, song);
                        }
                        ResponseType::UnsupportedSong {
                            path: unsupported_path,
                            msg,
                        } => {
                            self.song_in_progress.remove(&unsupported_path);
                            if self
                                .responses
                                .send(ResponseType::UnsupportedSong {
                                    path: unsupported_path.clone(),
                                    msg: msg,
                                })
                                .is_err()
                            {
                                return true;
                            }
                            if unsupported_path == key.0 || unsupported_path == key.1 {
                                break 'inner1;
                            }
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
        let path = if path.starts_with("file://") {
            //println!("path guess: `{}`", path.get(7..).unwrap());
            path.get(7..).unwrap().to_owned()
        } else if !path.contains("://") {
            path
        } else {
            if self
                .responses
                .send(ResponseType::UnsupportedSong {
                    msg: format!("Song path is not a supported URI, it's `{}`", path),
                    path: path,
                })
                .is_err()
            {
                return true;
            }
            return false;
        };
        if let Some(song) = self.song_cache.get(&path) {
            if ack {
                let song = song.to_owned();
                if self
                    .responses
                    .send(ResponseType::Song {
                        path: path,
                        song: song.map(Box::new),
                    })
                    .is_err()
                {
                    return true;
                }
            }
        } else {
            if !self.song_in_progress.contains(&path) {
                // every song is roughly 2 threads -- Song::from_path(...) spawns a thread
                let available_parallelism = std::thread::available_parallelism()
                    .ok()
                    .map(|x| x.get())
                    .unwrap_or(DEFAULT_PARALLELISM)
                    / 2;
                let available_parallelism = if available_parallelism != 0 {
                    available_parallelism - 1
                } else {
                    0
                };
                // wait for processing to complete if too many tasks already running
                if self.song_in_progress.len() > available_parallelism {
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
                                if self.song_in_progress.len() <= available_parallelism {
                                    break 'inner2;
                                }
                            }
                            ResponseType::UnsupportedSong { path, .. } => {
                                self.song_in_progress.remove(&path);
                                if self.song_in_progress.len() <= available_parallelism {
                                    break 'inner2;
                                }
                            }
                        }
                    }
                }
                let path_clone = path.clone();
                let results = worker_tx.clone();
                std::thread::spawn(move || {
                    let song_result = Song::from_path(&path_clone);
                    results
                        .send(ResponseType::Song {
                            path: path_clone,
                            song: song_result.map(Box::new),
                        })
                        .unwrap_or(());
                });
            }
            if ack {
                'inner3: for result in worker_results.iter() {
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
                                if self
                                    .responses
                                    .send(ResponseType::Song {
                                        path: path,
                                        song: song,
                                    })
                                    .is_err()
                                {
                                    return true;
                                }
                                break 'inner3;
                            }
                        }
                        ResponseType::UnsupportedSong {
                            path: unsupported_path,
                            msg,
                        } => {
                            self.song_in_progress.remove(&unsupported_path);
                            if unsupported_path == path {
                                if self
                                    .responses
                                    .send(ResponseType::UnsupportedSong {
                                        path: unsupported_path,
                                        msg: msg,
                                    })
                                    .is_err()
                                {
                                    return true;
                                }
                                break 'inner3;
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

#[cfg(feature = "bliss-audio-symphonia")]
fn worker_distance(
    results: &Sender<ResponseType>,
    song1: (&str, Option<Song>),
    song2: (&str, Option<Song>),
) -> Result<f32, BlissError> {
    let path1 = song1.0;
    let song1 = if let Some(song) = song1.1 {
        song
    } else {
        let new_song1 = Song::from_path(path1);
        results
            .send(ResponseType::Song {
                path: path1.to_string(),
                song: new_song1.clone().map(Box::new),
            })
            .unwrap_or(());
        new_song1?
    };
    let path2 = song2.0;
    let song2 = if let Some(song) = song2.1 {
        song
    } else {
        let new_song2 = Song::from_path(path2);
        results
            .send(ResponseType::Song {
                path: path2.to_string(),
                song: new_song2.clone().map(Box::new),
            })
            .unwrap_or(());
        /*if new_song2.is_err() {
            eprintln!(
                "Song error on `{}`: {}",
                path2,
                new_song2.clone().err().unwrap()
            );
        }*/
        new_song2?
    };
    Ok(song1.distance(&song2))
}
