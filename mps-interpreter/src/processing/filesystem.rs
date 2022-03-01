use std::fmt::{Debug, Display, Error, Formatter};
use std::fs::{DirEntry, ReadDir};
use std::iter::Iterator;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::lang::{MpsTypePrimitive, RuntimeMsg};
use crate::MpsItem;

const DEFAULT_REGEX: &str = r"/(?P<artist>[^/]+)/(?P<album>[^/]+)/(?:(?:(?P<disc>\d+)\s+)?(?P<track>\d+)\.?\s+)?(?P<title>[^/]+)\.(?P<format>(?:mp3)|(?:wav)|(?:ogg)|(?:flac)|(?:mp4)|(?:aac))$";

const DEFAULT_VEC_CACHE_SIZE: usize = 4;

#[derive(Debug)]
pub struct FileIter {
    root: PathBuf,
    pattern: Option<Regex>,
    tags_pattern: Regex,
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
            for dir in self.dir_iter.by_ref() {
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
            self.pattern
                .as_ref()
                .map(|re| re.to_string())
                .unwrap_or("[none]".to_string()),
            self.recursive
        )
    }
}

impl FileIter {
    pub fn new<P: AsRef<Path>>(
        root: Option<P>,
        pattern: Option<&str>,
        recurse: bool,
    ) -> Result<Self, String> {
        let root_path = match root {
            None => crate::lang::utility::music_folder(),
            Some(p) => p.as_ref().to_path_buf(),
        };
        let dir_vec = if root_path.is_dir() {
            let mut vec = Vec::with_capacity(DEFAULT_VEC_CACHE_SIZE);
            vec.push(
                root_path
                    .read_dir()
                    .map_err(|e| format!("Directory read error: {}", e))?
                    .into(),
            );
            vec
        } else {
            Vec::with_capacity(DEFAULT_VEC_CACHE_SIZE)
        };
        let pattern_re = if let Some(pattern) = pattern {
            Some(Regex::new(pattern).map_err(|e| format!("Regex compile error: {}", e))?)
        } else {
            None
        };
        let tags_re =
            Regex::new(DEFAULT_REGEX).map_err(|e| format!("Regex compile error: {}", e))?;
        Ok(Self {
            root: root_path,
            pattern: pattern_re,
            tags_pattern: tags_re,
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
            pattern: None,
            tags_pattern: Regex::new(DEFAULT_REGEX).unwrap(),
            recursive: recurse,
            dir_iters: dir_vec,
            is_complete: false,
        }
    }

    fn build_item<P: AsRef<Path>>(&self, filepath: P) -> Option<MpsItem> {
        let path = filepath.as_ref();
        let path_str = path.to_str()?;
        #[cfg(debug_assertions)]
        if !path.is_file() {
            panic!("Got non-file path `{}` when building music item", path_str)
        }
        if let Some(pattern) = &self.pattern {
            let captures = pattern.captures(path_str)?;
            let capture_names = pattern.capture_names();
            // populate fields
            self.populate_item_impl(path, path_str, Some(captures), capture_names)
        } else {
            let captures = self.tags_pattern.captures(path_str);
            let capture_names = self.tags_pattern.capture_names();
            self.populate_item_impl(path, path_str, captures, capture_names)
        }
    }

    #[cfg(feature = "music_library")]
    fn populate_item_impl(
        &self,
        path: &Path,
        path_str: &str,
        captures: Option<regex::Captures>,
        capture_names: regex::CaptureNames,
    ) -> Option<MpsItem> {
        match crate::music::MpsLibrary::read_media_tags(path) {
            Ok(tags) => {
                let mut item = MpsItem::new();
                self.populate_item_impl_simple(&mut item, path_str, captures, capture_names);
                if item.field("title").is_none() {
                    item.set_field("title", tags.track_title().into());
                }
                if item.field("artist").is_none() {
                    if let Some(artist) = tags.artist_name() {
                        item.set_field("artist", artist.into());
                    }
                }
                if item.field("album").is_none() {
                    if let Some(album) = tags.album_title() {
                        item.set_field("album", album.into());
                    }
                }
                if item.field("genre").is_none() {
                    if let Some(genre) = tags.genre_title() {
                        item.set_field("genre", genre.into());
                    }
                }
                if item.field("track").is_none() {
                    if let Some(track) = tags.track_number() {
                        item.set_field("track", track.into());
                    }
                }
                if item.field("year").is_none() {
                    if let Some(year) = tags.track_date() {
                        item.set_field("year", year.into());
                    }
                }
                Some(item)
            }
            Err(_) => {
                let mut item = MpsItem::new();
                self.populate_item_impl_simple(&mut item, path_str, captures, capture_names);
                Some(item)
            }
        }
    }

    #[cfg(not(feature = "music_library"))]
    fn populate_item_impl(
        &self,
        path_str: &str,
        captures: Option<regex::Captures>,
        capture_names: regex::CaptureNames,
    ) -> Option<MpsItem> {
        let mut item = MpsItem::new();
        self.populate_item_impl_simple(&mut item, path_str, captures, capture_names);
        Some(item)
    }

    #[inline]
    fn populate_item_impl_simple(
        &self,
        item: &mut MpsItem,
        path_str: &str,
        captures: Option<regex::Captures>,
        capture_names: regex::CaptureNames,
    ) {
        // populates fields from named capture groups
        if let Some(captures) = captures {
            for name_maybe in capture_names {
                if let Some(name) = name_maybe {
                    if let Some(value) = captures.name(name).map(|m| m.as_str().to_string()) {
                        item.set_field(name, MpsTypePrimitive::parse(value));
                    }
                }
            }
        }
        item.set_field("filename", path_str.to_string().into());
    }

    fn only_once(&mut self) -> Result<MpsItem, String> {
        if self.root.is_file() {
            self.is_complete = true;
            match self.build_item(&self.root) {
                Some(item) => Ok(item),
                None => Err(format!(
                    "Failed to populate item from file `{}`",
                    self.root.display()
                )),
            }
        } else {
            Err(format!(
                "Cannot populate item from non-file `{}`",
                self.root.display()
            ))
        }
    }

    /*fn default_title(path: &Path) -> String {
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        path.file_name()
            .and_then(|file| file.to_str())
            .and_then(|file| Some(file.replacen(&format!(".{}", extension), "", 1)))
            .unwrap_or("Unknown Title".into())
    }*/
}

impl Iterator for FileIter {
    type Item = Result<MpsItem, String>;

    //#[recursion_limit = "1024"]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_complete {
            None
        } else if self.dir_iters.is_empty() {
            if self.root.is_file() {
                self.is_complete = true;
                self.build_item(&self.root).map(Ok)
            } else {
                self.dir_iters.push(match self.root.read_dir() {
                    Ok(x) => x.into(),
                    Err(e) => {
                        self.is_complete = true;
                        return Some(Err(format!("Directory read error: {}", e)));
                    }
                });
                self.next()
            }
        } else {
            while !self.dir_iters.is_empty() {
                let mut dir_iter = self.dir_iters.pop().unwrap();
                'inner: while let Some(path_result) = dir_iter.next() {
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
                                    //return self.next();
                                    break 'inner;
                                }
                            } else if let Some(item) = self.build_item(dir_entry.path()) {
                                self.dir_iters.push(dir_iter);
                                return Some(Ok(item));
                            }
                        }
                        Err(e) => {
                            self.dir_iters.push(dir_iter);
                            return Some(Err(format!("Path read error: {}", e)));
                        }
                    }
                }
            }
            self.is_complete = true;
            None
        }
    }
}

pub trait MpsFilesystemQuerier: Debug {
    fn raw(
        &mut self,
        folder: Option<&str>,
        pattern: Option<&str>,
        recursive: bool,
    ) -> Result<FileIter, RuntimeMsg>;

    fn single(&mut self, path: &str, pattern: Option<&str>) -> Result<MpsItem, RuntimeMsg>;

    fn expand(&self, folder: Option<&str>) -> Result<Option<String>, RuntimeMsg> {
        #[cfg(feature = "shellexpand")]
        match folder {
            Some(path) => Ok(Some(
                shellexpand::full(path)
                    .map_err(|e| RuntimeMsg(format!("Path expansion error: {}", e)))?
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
    ) -> Result<FileIter, RuntimeMsg> {
        let folder = self.expand(folder)?;
        FileIter::new(folder, pattern, recursive).map_err(RuntimeMsg)
    }

    fn single(&mut self, path: &str, pattern: Option<&str>) -> Result<MpsItem, RuntimeMsg> {
        let path = self.expand(Some(path))?;
        let mut file_iter = FileIter::new(path, pattern, false).map_err(RuntimeMsg)?;
        file_iter.only_once().map_err(RuntimeMsg)
    }
}
