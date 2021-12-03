use std::collections::HashMap;
use std::path::Path;

use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

use crate::lang::db::*;
use super::tag::Tags;

#[derive(Clone, Default)]
pub struct MpsLibrary {
    songs: HashMap<u64, DbMusicItem>,
    metadata: HashMap<u64, DbMetaItem>,
    artists: HashMap<String, DbArtistItem>,
    albums: HashMap<String, DbAlbumItem>,
    genres: HashMap<String, DbGenreItem>,
}

impl MpsLibrary {
    pub fn new() -> Self {
        Self {
            songs: HashMap::new(),
            metadata: HashMap::new(),
            artists: HashMap::new(),
            albums: HashMap::new(),
            genres: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.songs.len()
    }

    pub fn all_songs<'a>(&'a self) -> Vec<&'a DbMusicItem> {
        self.songs.values().collect()
    }

    pub fn all_metadata<'a>(&'a self) -> Vec<&'a DbMetaItem> {
        self.metadata.values().collect()
    }

    pub fn all_artists<'a>(&'a self) -> Vec<&'a DbArtistItem> {
        self.artists.values().collect()
    }

    pub fn all_albums<'a>(&'a self) -> Vec<&'a DbAlbumItem> {
        self.albums.values().collect()
    }

    pub fn all_genres<'a>(&'a self) -> Vec<&'a DbGenreItem> {
        self.genres.values().collect()
    }

    pub fn read_path<P: AsRef<Path>>(&mut self, path: P, depth: usize) -> std::io::Result<()> {
        let path = path.as_ref();
        if path.is_dir() && depth != 0 {
            for entry in path.read_dir()? {
                self.read_path(entry?.path(), depth-1)?;
            }
        } else if path.is_file() {
            self.read_file(path)?;
        }
        Ok(())
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
            &Default::default()
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
        if tags.len() == 0 { return; } // probably not a valid song, let's skip it
        let song_id = self.songs.len() as u64; // guaranteed to be created
        let meta_id = self.metadata.len() as u64; // guaranteed to be created
        self.metadata.insert(meta_id, tags.meta(meta_id)); // definitely necessary
        // genre has no links to others, so find that first
        let mut genre = tags.genre(0);
        genre.genre_id = Self::find_or_gen_id(&self.genres, &genre.title);
        if genre.genre_id == self.genres.len() as u64 {
            self.genres.insert(Self::sanitise_key(&genre.title), genre.clone());
        }
        // artist only links to genre, so that can be next
        let mut artist = tags.artist(0, genre.genre_id);
        artist.artist_id = Self::find_or_gen_id(&self.artists, &artist.name);
        if artist.artist_id == self.artists.len() as u64 {
            self.artists.insert(Self::sanitise_key(&artist.name), artist.clone());
        }
        // same with album artist
        let mut album_artist = tags.album_artist(0, genre.genre_id);
        album_artist.artist_id = Self::find_or_gen_id(&self.artists, &album_artist.name);
        if album_artist.artist_id == self.artists.len() as u64 {
            self.artists.insert(Self::sanitise_key(&album_artist.name), album_artist.clone());
        }
        // album now has all links ready
        let mut album = tags.album(0, 0, album_artist.artist_id, genre.genre_id);
        album.album_id = Self::find_or_gen_id(&self.albums, &album.title);
        if album.album_id == self.albums.len() as u64 {
            let album_meta = tags.album_meta(self.metadata.len() as u64);
            album.metadata = album_meta.meta_id;
            self.albums.insert(Self::sanitise_key(&album.title), album.clone());
            self.metadata.insert(album_meta.meta_id, album_meta);
        }
        //let meta_album_id = self.metadata.len() as u64;
        //let album = tags.album(album_id, meta_album_id);
        self.songs.insert(song_id, tags.song(song_id, artist.artist_id, Some(album.album_id), meta_id, genre.genre_id));
    }

    #[inline]
    fn find_or_gen_id<'a, D: DatabaseObj>(map: &'a HashMap<String, D>, key: &str) -> u64 {
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
