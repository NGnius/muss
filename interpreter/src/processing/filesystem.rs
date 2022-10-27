use std::fmt::{Debug, Display, Error, Formatter};
use std::fs::{DirEntry, ReadDir};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::io::Read;

use regex::Regex;

use crate::lang::{RuntimeMsg, TypePrimitive, GeneratorOp};
use crate::Item;

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
            self.cache
                .sort_by_key(|b| std::cmp::Reverse(b.path().to_string_lossy().to_lowercase()));
            /*self.cache.sort_by(
                |a, b| b.path().to_string_lossy().to_lowercase().cmp(
                    &a.path().to_string_lossy().to_lowercase())
            );*/
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
                .unwrap_or_else(|| "[none]".to_string()),
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

    fn build_item<P: AsRef<Path>>(&self, filepath: P) -> Option<Item> {
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
    ) -> Option<Item> {
        match crate::music::Library::read_media_tags(path) {
            Ok(tags) => {
                let mut item = Item::new();
                tags.export_to_item(&mut item, true);
                /*item.set_field("title", tags.track_title().into());
                if let Some(artist) = tags.artist_name() {
                    item.set_field("artist", artist.into());
                }
                if let Some(albumartist) = tags.albumartist_name() {
                    item.set_field("albumartist", albumartist.clone().into());
                    if let Some(TypePrimitive::String(artist)) = item.field("artist") {
                        if albumartist.trim() != artist.trim() {
                            let new_artist = format!("{},{}", artist, albumartist.as_str());
                            item.set_field("artist", new_artist.into());
                        }
                    } else {
                        item.set_field("artist", albumartist.into());
                    }
                }
                if let Some(album) = tags.album_title() {
                    item.set_field("album", album.into());
                }
                if let Some(genre) = tags.genre_title() {
                    item.set_field("genre", genre.into());
                }
                if let Some(track) = tags.track_number() {
                    item.set_field("track", track.into());
                }
                if let Some(year) = tags.track_date() {
                    item.set_field("year", year.into());
                }
                if let Some(cover) = tags.cover_art() {
                    item.set_field("cover", cover.into());
                }*/
                self.populate_item_impl_simple(&mut item, path_str, captures, capture_names);
                Some(item)
            }
            Err(_) => {
                let mut item = Item::new();
                self.populate_item_impl_simple(&mut item, path_str, captures, capture_names);
                Some(item)
            }
        }
    }

    #[cfg(not(feature = "music_library"))]
    fn populate_item_impl(
        &self,
        _path: &Path,
        path_str: &str,
        captures: Option<regex::Captures>,
        capture_names: regex::CaptureNames,
    ) -> Option<Item> {
        let mut item = Item::new();
        self.populate_item_impl_simple(&mut item, path_str, captures, capture_names);
        Some(item)
    }

    #[inline]
    fn populate_item_impl_simple(
        &self,
        item: &mut Item,
        path_str: &str,
        captures: Option<regex::Captures>,
        capture_names: regex::CaptureNames,
    ) {
        // populates fields from named capture groups
        if let Some(captures) = captures {
            for name in capture_names.flatten() {
                if item.field(name).is_some() {
                    // do nothing
                } else if let Some(value) = captures.name(name).map(|m| m.as_str().to_string()) {
                    item.set_field(name, TypePrimitive::parse(value));
                }
            }
        }
        item.set_field("filename", format!("file://{}", path_str).into());
    }

    fn only_once(&mut self) -> Result<Item, String> {
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
    type Item = Result<Item, String>;

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

pub trait FilesystemQuerier: Debug {
    fn raw(
        &mut self,
        folder: Option<&str>,
        pattern: Option<&str>,
        recursive: bool,
    ) -> Result<FileIter, RuntimeMsg>;

    fn single(&mut self, path: &str, pattern: Option<&str>) -> Result<Item, RuntimeMsg>;

    fn read_file(&mut self, path: &str) -> Result<GeneratorOp, RuntimeMsg>;

    fn expand(&self, folder: Option<&str>) -> Result<Option<String>, RuntimeMsg> {
        #[cfg(feature = "shellexpand")]
        match folder {
            Some(path) => Ok(Some(
                shellexpand::full(self.canonicalize(path))
                    .map_err(|e| RuntimeMsg(format!("Path expansion error: {}", e)))?
                    .into_owned(),
            )),
            None => Ok(None),
        }
        #[cfg(not(feature = "shellexpand"))]
        Ok(folder.and_then(|s| Some(self.canonicalize(s).to_string())))
    }

    fn canonicalize<'a>(&self, path: &'a str) -> &'a str {
        if let Some(new_path) = path.strip_prefix("file://") {
            new_path
        } else {
            path
        }
    }
}

#[derive(Default, Debug)]
pub struct FilesystemExecutor {}

impl FilesystemExecutor {
    #[cfg(feature = "collections")]
    fn read_m3u8<P: AsRef<Path> + 'static>(&self, path: P) -> Result<GeneratorOp, RuntimeMsg> {
        let mut file = std::fs::File::open(&path).map_err(|e| RuntimeMsg(format!("Path read error: {}", e)))?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes).map_err(|e| RuntimeMsg(format!("File read error: {}", e)))?;
        let (_, playlist) = m3u8_rs::parse_playlist(&file_bytes).map_err(|e| RuntimeMsg(format!("Playlist read error: {}", e)))?;
        let playlist = match playlist {
            m3u8_rs::Playlist::MasterPlaylist(_) => return Err(RuntimeMsg(format!("Playlist not supported: `{}` is a master (not media) playlist", path.as_ref().display()))),
            m3u8_rs::Playlist::MediaPlaylist(l) => l,
        };
        let mut index = 0;
        Ok(GeneratorOp::new(move |ctx| {
            if let Some(segment) = playlist.segments.get(index) {
                let path = path.as_ref();
                index += 1;
                let item_path = if let Some(parent) = path.parent() {
                    let joined_path = parent.join(&segment.uri);
                    if let Some(s) = joined_path.to_str() {
                        s.to_owned()
                    } else {
                        return Some(Err(RuntimeMsg(format!("Failed to convert path to string for `{}`", joined_path.display()))));
                    }
                } else {
                    segment.uri.clone()
                };
                let item = match ctx.filesystem.single(&item_path, None) {
                    Err(e) => Err(e),
                    Ok(mut item) => {
                        item.set_field("duration", (segment.duration as f64).into());
                        if let Some(title) = &segment.title {
                            item.set_field("title", title.to_owned().into());
                        }
                        Ok(item)
                    }
                };
                Some(item)
            } else {
                None
            }
        }))
    }
}

impl FilesystemQuerier for FilesystemExecutor {
    fn raw(
        &mut self,
        folder: Option<&str>,
        pattern: Option<&str>,
        recursive: bool,
    ) -> Result<FileIter, RuntimeMsg> {
        let folder = self.expand(folder)?;
        FileIter::new(folder, pattern, recursive).map_err(RuntimeMsg)
    }

    fn single(&mut self, path: &str, pattern: Option<&str>) -> Result<Item, RuntimeMsg> {
        let path = self.expand(Some(path))?;
        let mut file_iter = FileIter::new(path, pattern, false).map_err(RuntimeMsg)?;
        file_iter.only_once().map_err(RuntimeMsg)
    }

    fn read_file(&mut self, path: &str) -> Result<GeneratorOp, RuntimeMsg> {
        let path: PathBuf = self.expand(Some(path))?.unwrap().into();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            match &ext.to_lowercase() as &str {
                #[cfg(feature = "collections")]
                "m3u8" => self.read_m3u8(path),
                ext => Err(RuntimeMsg(format!("Unrecognised extension `{}` in path `{}`", ext, path.display())))
            }
        } else {
            Err(RuntimeMsg(format!("Unrecognised path `{}`", path.display())))
        }
    }
}
