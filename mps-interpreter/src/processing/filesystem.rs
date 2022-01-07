use std::fmt::{Debug, Display, Error, Formatter};
use std::fs::{ReadDir, DirEntry};
use std::iter::Iterator;
use std::path::{Path, PathBuf};

use regex::Regex;

use super::OpGetter;
use crate::lang::RuntimeError;
use crate::MpsMusicItem;

const DEFAULT_REGEX: &str = r"/(?P<artist>[^/]+)/(?P<album>[^/]+)/(?:(?:(?P<disc>\d+)\s+)?(?P<track>\d+)\.?\s+)?(?P<title>[^/]+)\.(?P<format>(?:mp3)|(?:wav)|(?:ogg)|(?:flac)|(?:mp4)|(?:aac))$";

const DEFAULT_VEC_CACHE_SIZE: usize = 4;

#[derive(Debug)]
pub struct FileIter {
    root: PathBuf,
    pattern: Regex,
    recursive: bool,
    dir_iters: Vec<SortedReadDir>,
    is_complete: bool,
}

#[derive(Debug)]
struct SortedReadDir {
    dir_iter: ReadDir,
    dir_iter_complete: bool,
    cache: Vec<DirEntry>,
}

impl Iterator for SortedReadDir {
    type Item = std::io::Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.dir_iter_complete {
            while let Some(dir) = self.dir_iter.next() {
                match dir {
                    Ok(f) => self.cache.push(f),
                    Err(e) => return Some(Err(e)),
                }
            }
            self.dir_iter_complete = true;
            self.cache.sort_by(|a, b| b.path().cmp(&a.path()));
        }
        if self.cache.is_empty() {
            None
        } else {
            Some(Ok(self.cache.pop().unwrap()))
        }
    }
}

impl std::convert::From<ReadDir> for SortedReadDir {
    fn from(item: ReadDir) -> Self {
        Self {
            dir_iter: item,
            dir_iter_complete: false,
            cache: Vec::new(),
        }
    }
}

impl Display for FileIter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "root=`{}`, pattern={}, recursive={}",
            self.root.to_str().unwrap_or(""),
            self.pattern,
            self.recursive
        )
    }
}

impl FileIter {
    pub fn new<P: AsRef<Path>>(
        root: Option<P>,
        pattern: Option<&str>,
        recurse: bool,
        op: &mut OpGetter,
    ) -> Result<Self, RuntimeError> {
        let root_path = match root {
            None => crate::lang::utility::music_folder(),
            Some(p) => p.as_ref().to_path_buf(),
        };
        let dir_vec = if root_path.is_dir() {
            let mut vec = Vec::with_capacity(DEFAULT_VEC_CACHE_SIZE);
            vec.push(root_path.read_dir().map_err(|e| RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Directory read error: {}", e),
            })?.into());
            vec
        } else {
            Vec::with_capacity(DEFAULT_VEC_CACHE_SIZE)
        };
        Ok(Self {
            root: root_path,
            pattern: Regex::new(pattern.unwrap_or(DEFAULT_REGEX)).map_err(|e| RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Regex compile error: {}", e),
            })?,
            recursive: recurse,
            dir_iters: dir_vec,
            is_complete: false,
        })
    }

    pub fn common_defaults(recurse: bool) -> Self {
        let root_path = crate::lang::utility::music_folder();
        let read_dir = root_path.read_dir().unwrap();
        let mut dir_vec = Vec::with_capacity(DEFAULT_VEC_CACHE_SIZE);
        dir_vec.push(read_dir.into());
        Self {
            root: root_path,
            pattern: Regex::new(DEFAULT_REGEX).unwrap(),
            recursive: recurse,
            dir_iters: dir_vec,
            is_complete: false,
        }
    }

    fn build_item<P: AsRef<Path>>(&self, filepath: P) -> Option<MpsMusicItem> {
        let path = filepath.as_ref();
        let path_str = path.to_str()?;
        #[cfg(debug_assertions)]
        if !path.is_file() {
            panic!("Got non-file path `{}` when building music item", path_str)
        }
        let captures = self.pattern.captures(path_str)?;
        // populate fields
        self.populate_item_impl(path, path_str, captures)
    }

    #[cfg(feature = "music_library")]
    fn populate_item_impl(
        &self,
        path: &Path,
        path_str: &str,
        captures: regex::Captures,
    ) -> Option<MpsMusicItem> {
        match crate::music::MpsLibrary::read_media_tags(path) {
            Ok(tags) => Some(MpsMusicItem {
                title: captures
                    .name("title")
                    .and_then(|m| Some(m.as_str().to_string()))
                    .unwrap_or_else(|| tags.track_title()),
                artist: captures
                    .name("artist")
                    .and_then(|m| Some(m.as_str().to_string()))
                    .or_else(|| tags.artist_name()),
                album: captures
                    .name("album")
                    .and_then(|m| Some(m.as_str().to_string()))
                    .or_else(|| tags.album_title()),
                filename: path_str.to_string(),
                genre: captures
                    .name("genre")
                    .and_then(|m| Some(m.as_str().to_string()))
                    .or_else(|| tags.genre_title()),
                track: match captures.name("track") {
                    None => tags.track_number(),
                    Some(m) => match m.as_str().parse::<u64>() {
                        Ok(u) => Some(u),
                        Err(_) => tags.track_number(),
                    },
                },
                year: match captures.name("year") {
                    None => tags.track_date(),
                    Some(m) => match m.as_str().parse::<u64>() {
                        Ok(u) => Some(u),
                        Err(_) => tags.track_date(),
                    },
                },
            }),
            Err(_) => self.populate_item_impl_simple(path, path_str, captures),
        }
    }

    #[cfg(not(feature = "music_library"))]
    fn populate_item_impl(
        &self,
        path: &Path,
        path_str: &str,
        captures: regex::Captures,
    ) -> Option<MpsMusicItem> {
        self.populate_item_impl_simple(path, path_str, captures)
    }

    #[inline]
    fn populate_item_impl_simple(
        &self,
        path: &Path,
        path_str: &str,
        captures: regex::Captures,
    ) -> Option<MpsMusicItem> {
        Some(MpsMusicItem {
            title: captures
                .name("title")
                .and_then(|m| Some(m.as_str().to_string()))
                .unwrap_or_else(|| Self::default_title(path)),
            artist: captures
                .name("artist")
                .and_then(|m| Some(m.as_str().to_string())),
            album: captures
                .name("album")
                .and_then(|m| Some(m.as_str().to_string())),
            filename: path_str.to_string(),
            genre: captures
                .name("genre")
                .and_then(|m| Some(m.as_str().to_string())),
            track: match captures.name("track") {
                None => None,
                Some(m) => match m.as_str().parse::<u64>() {
                    Ok(u) => Some(u),
                    Err(_) => None,
                },
            },
            year: match captures.name("year") {
                None => None,
                Some(m) => match m.as_str().parse::<u64>() {
                    Ok(u) => Some(u),
                    Err(_) => None,
                },
            },
        })
    }

    fn default_title(path: &Path) -> String {
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        path.file_name()
            .and_then(|file| file.to_str())
            .and_then(|file| Some(file.replacen(&format!(".{}", extension), "", 1)))
            .unwrap_or("Unknown Title".into())
    }
}

impl Iterator for FileIter {
    type Item = Result<MpsMusicItem, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_complete {
            None
        } else {
            if self.dir_iters.is_empty() {
                if self.root.is_file() {
                    self.is_complete = true;
                    match self.build_item(&self.root) {
                        None => None,
                        Some(item) => Some(Ok(item)),
                    }
                } else {
                    self.dir_iters.push(match self.root.read_dir() {
                        Ok(x) => x.into(),
                        Err(e) => {
                            self.is_complete = true;
                            return Some(Err(format!("Directory read error: {}", e)));
                        },
                    });
                    return self.next();
                }
            } else {
                while !self.dir_iters.is_empty() {
                    let mut dir_iter = self.dir_iters.pop().unwrap();
                    while let Some(path_result) = dir_iter.next() {
                        match path_result {
                            Ok(dir_entry) => {
                                if dir_entry.path().is_dir() {
                                    if self.recursive {
                                        self.dir_iters.push(dir_iter);
                                        self.dir_iters.push(match dir_entry.path().read_dir() {
                                            Ok(x) => x.into(),
                                            Err(e) => {
                                                return Some(Err(format!(
                                                    "Directory read error: {}",
                                                    e
                                                )))
                                            }
                                        });
                                        return self.next();
                                    }
                                } else {
                                    if let Some(item) = self.build_item(dir_entry.path()) {
                                        self.dir_iters.push(dir_iter);
                                        return Some(Ok(item));
                                    }
                                }
                            }
                            Err(e) => {
                                self.dir_iters.push(dir_iter);
                                return Some(Err(format!("Path read error: {}", e)));
                            }
                        }
                    }
                }
                None
            }
        }
    }
}

pub trait MpsFilesystemQuerier: Debug {
    fn raw(
        &mut self,
        folder: Option<&str>,
        pattern: Option<&str>,
        recursive: bool,
        op: &mut OpGetter,
    ) -> Result<FileIter, RuntimeError>;

    fn expand(
        &self,
        folder: Option<&str>,
        #[allow(unused_variables)] op: &mut OpGetter,
    ) -> Result<Option<String>, RuntimeError> {
        #[cfg(feature = "shellexpand")]
        match folder {
            Some(path) => Ok(Some(
                shellexpand::full(path)
                    .map_err(|e| RuntimeError {
                        line: 0,
                        op: op(),
                        msg: format!("Path expansion error: {}", e),
                    })?
                    .into_owned(),
            )),
            None => Ok(None),
        }
        #[cfg(not(feature = "shellexpand"))]
        Ok(folder.and_then(|s| Some(s.to_string())))
    }
}

#[derive(Default, Debug)]
pub struct MpsFilesystemExecutor {}

impl MpsFilesystemQuerier for MpsFilesystemExecutor {
    fn raw(
        &mut self,
        folder: Option<&str>,
        pattern: Option<&str>,
        recursive: bool,
        op: &mut OpGetter,
    ) -> Result<FileIter, RuntimeError> {
        let folder = self.expand(folder, op)?;
        FileIter::new(folder, pattern, recursive, op)
    }
}
