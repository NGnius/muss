use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

use super::tag::Tags;
use crate::lang::db::*;

#[derive(Clone, Default)]
pub struct Library {
    songs: HashMap<u64, DbMusicItem>,
    metadata: HashMap<u64, DbMetaItem>,
    artists: HashMap<String, DbArtistItem>,
    albums: HashMap<String, DbAlbumItem>,
    genres: HashMap<String, DbGenreItem>,
    files: HashSet<PathBuf>,
    dirty: bool,
}

impl Library {
    pub fn new() -> Self {
        Self {
            songs: HashMap::new(),
            metadata: HashMap::new(),
            artists: HashMap::new(),
            albums: HashMap::new(),
            genres: HashMap::new(),
            files: HashSet::new(),
            dirty: false,
        }
    }

    pub fn len(&self) -> usize {
        self.songs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.songs.is_empty()
    }

    pub fn clear_modified(&mut self) {
        self.dirty = false;
    }

    pub fn is_modified(&self) -> bool {
        self.dirty
    }

    #[inline(always)]
    fn modify(&mut self) {
        self.dirty = true;
    }

    #[inline]
    pub fn contains_path<P: AsRef<Path>>(&self, path: P) -> bool {
        self.files.contains(&path.as_ref().to_path_buf())
    }

    pub fn all_songs(&self) -> Vec<&'_ DbMusicItem> {
        self.songs.values().collect()
    }

    #[inline]
    pub fn add_song(&mut self, song: DbMusicItem) {
        self.modify();
        if let Ok(path) = PathBuf::from_str(&song.filename) {
            self.files.insert(path);
        }
        self.songs.insert(song.song_id, song);
    }

    pub fn all_metadata(&self) -> Vec<&'_ DbMetaItem> {
        self.metadata.values().collect()
    }

    #[inline]
    pub fn add_metadata(&mut self, meta: DbMetaItem) {
        self.modify();
        self.metadata.insert(meta.meta_id, meta);
    }

    pub fn all_artists(&self) -> Vec<&'_ DbArtistItem> {
        self.artists.values().collect()
    }

    #[inline]
    pub fn add_artist(&mut self, artist: DbArtistItem) {
        self.modify();
        self.artists
            .insert(Self::sanitise_key(&artist.name), artist);
    }

    pub fn all_albums(&self) -> Vec<&'_ DbAlbumItem> {
        self.albums.values().collect()
    }

    #[inline]
    pub fn add_album(&mut self, album: DbAlbumItem) {
        self.modify();
        self.albums.insert(Self::sanitise_key(&album.title), album);
    }

    pub fn all_genres(&self) -> Vec<&'_ DbGenreItem> {
        self.genres.values().collect()
    }

    #[inline]
    pub fn add_genre(&mut self, genre: DbGenreItem) {
        self.modify();
        self.genres.insert(Self::sanitise_key(&genre.title), genre);
    }

    pub fn read_path<P: AsRef<Path>>(&mut self, path: P, depth: usize) -> std::io::Result<()> {
        let path = path.as_ref();
        if self.contains_path(path) {
            return Ok(());
        } // skip existing entries
        if path.is_dir() && depth != 0 {
            for entry in path.read_dir()? {
                self.read_path(entry?.path(), depth - 1)?;
            }
        } else if path.is_file() {
            self.read_file(path)?;
        }
        Ok(())
    }

    pub fn read_media_tags<P: AsRef<Path>>(path: P) -> std::io::Result<Tags> {
        let path = path.as_ref();
        let file = Box::new(std::fs::File::open(path)?);
        // use symphonia to get metadata
        let mss = MediaSourceStream::new(file, Default::default() /* options */);
        let probed = symphonia::default::get_probe().format(
            &Hint::new(),
            mss,
            &Default::default(),
            &Default::default(),
        );
        let mut tags = Tags::new(path);
        if let Ok(mut probed) = probed {
            // collect metadata
            if let Some(metadata) = probed.metadata.get() {
                if let Some(rev) = metadata.current() {
                    for tag in rev.tags() {
                        //println!("(pre) metadata tag ({},{})", tag.key, tag.value);
                        tags.add(tag.key.clone(), &tag.value);
                    }
                }
            }
            if let Some(rev) = probed.format.metadata().current() {
                for tag in rev.tags() {
                    //println!("(post) metadata tag ({},{})", tag.key, tag.value);
                    tags.add(tag.key.clone(), &tag.value);
                }
            }
        }
        Ok(tags)
    }

    fn read_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        let file = Box::new(std::fs::File::open(path)?);
        // use symphonia to get metadata
        let mss = MediaSourceStream::new(file, Default::default() /* options */);
        let probed = symphonia::default::get_probe().format(
            &Hint::new(),
            mss,
            &Default::default(),
            &Default::default(),
        );
        // process audio file, ignoring any processing errors (skip file on error)
        if let Ok(mut probed) = probed {
            let mut tags = Tags::new(path);
            // collect metadata
            if let Some(metadata) = probed.metadata.get() {
                if let Some(rev) = metadata.current() {
                    for tag in rev.tags() {
                        //println!("(pre) metadata tag ({},{})", tag.key, tag.value);
                        tags.add(tag.key.clone(), &tag.value);
                    }
                }
            }
            if let Some(rev) = probed.format.metadata().current() {
                for tag in rev.tags() {
                    //println!("(post) metadata tag ({},{})", tag.key, tag.value);
                    tags.add(tag.key.clone(), &tag.value);
                }
            }
            self.generate_entries(&tags);
        }
        Ok(())
    }

    /// generate data structures and links
    fn generate_entries(&mut self, tags: &Tags) {
        if tags.len() == 0 {
            return;
        } // probably not a valid song, let's skip it
        let song_id = self.songs.len() as u64; // guaranteed to be created
        let meta_id = self.metadata.len() as u64; // guaranteed to be created
        self.add_metadata(tags.meta(meta_id)); // definitely necessary
                                               // genre has no links to others, so find that first
        let mut genre = tags.genre(0);
        genre.genre_id = Self::find_or_gen_id(&self.genres, &genre.title);
        if genre.genre_id == self.genres.len() as u64 {
            self.add_genre(genre.clone());
        }
        // artist only links to genre, so that can be next
        let mut artist = tags.artist(0, genre.genre_id);
        artist.artist_id = Self::find_or_gen_id(&self.artists, &artist.name);
        if artist.artist_id == self.artists.len() as u64 {
            self.add_artist(artist.clone());
        }
        // same with album artist
        let mut album_artist = tags.album_artist(0, genre.genre_id);
        album_artist.artist_id = Self::find_or_gen_id(&self.artists, &album_artist.name);
        if album_artist.artist_id == self.artists.len() as u64 {
            self.add_artist(album_artist.clone());
        }
        // album now has all links ready
        let mut album = tags.album(0, 0, album_artist.artist_id, genre.genre_id);
        album.album_id = Self::find_or_gen_id(&self.albums, &album.title);
        if album.album_id == self.albums.len() as u64 {
            let album_meta = tags.album_meta(self.metadata.len() as u64);
            album.metadata = album_meta.meta_id;
            self.add_album(album.clone());
            self.add_metadata(album_meta);
        }
        //let meta_album_id = self.metadata.len() as u64;
        //let album = tags.album(album_id, meta_album_id);
        self.add_song(tags.song(
            song_id,
            artist.artist_id,
            Some(album.album_id),
            meta_id,
            genre.genre_id,
        ));
    }

    #[inline]
    fn find_or_gen_id<D: DatabaseObj>(map: &HashMap<String, D>, key: &str) -> u64 {
        if let Some(obj) = Self::find_by_key(map, key) {
            obj.id()
        } else {
            map.len() as u64
        }
    }

    #[inline(always)]
    fn find_by_key<'a, D: DatabaseObj>(map: &'a HashMap<String, D>, key: &str) -> Option<&'a D> {
        map.get(&Self::sanitise_key(key))
    }

    #[inline(always)]
    fn sanitise_key(key: &str) -> String {
        key.trim().to_lowercase()
    }
}
