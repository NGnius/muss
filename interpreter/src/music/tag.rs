use std::collections::HashMap;
use std::path::{Path, PathBuf};

use symphonia::core::meta::Value;

use crate::lang::db::*;

pub struct Tags {
    data: HashMap<String, TagType>,
    filename: PathBuf,
}

impl Tags {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            data: HashMap::new(),
            filename: path.as_ref().canonicalize().unwrap(),
        }
    }

    pub fn add(&mut self, key: String, value: &Value) {
        if let Some(tag_type) = TagType::from_symphonia_value(value) {
            self.data.insert(key.trim().to_uppercase(), tag_type);
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn track_title(&self) -> String {
        self.data
            .get("TITLE")
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.default_title())
    }

    #[inline]
    fn default_title(&self) -> String {
        let extension = self
            .filename
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        self.filename
            .file_name()
            .and_then(|file| file.to_str())
            .map(|file| file.replacen(&format!(".{}", extension), "", 1))
            .unwrap_or_else(|| "Unknown Title".into())
    }

    #[inline]
    pub fn artist_name(&self) -> Option<String> {
        self.data
            .get("ARTIST")
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn album_title(&self) -> Option<String> {
        self.data
            .get("ALBUM")
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn albumartist_name(&self) -> Option<String> {
        self.data
            .get("ALBUMARTIST")
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn genre_title(&self) -> Option<String> {
        self.data
            .get("GENRE")
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn track_number(&self) -> Option<u64> {
        self.data
            .get("TRACKNUMBER")
            .unwrap_or(&TagType::Unknown)
            .uint()
    }

    #[inline]
    pub fn track_date(&self) -> Option<u64> {
        self.data.get("DATE").unwrap_or(&TagType::Unknown).uint()
    }

    pub fn song(
        &self,
        id: u64,
        artist_id: u64,
        album_id: Option<u64>,
        meta_id: u64,
        genre_id: u64,
    ) -> DbMusicItem {
        DbMusicItem {
            song_id: id,
            title: self.track_title(),
            artist: artist_id,
            album: album_id,
            filename: format!("file://{}", self.filename.to_str().unwrap_or("")),
            metadata: meta_id,
            genre: genre_id,
        }
    }

    pub fn meta(&self, id: u64) -> DbMetaItem {
        DbMetaItem {
            meta_id: id,
            plays: self
                .data
                .get("PLAYS")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
            track: self.track_number().unwrap_or(id),
            disc: self
                .data
                .get("DISCNUMBER")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(1),
            duration: self
                .data
                .get("DURATION")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
            date: self.track_date().unwrap_or(0),
        }
    }

    pub fn artist(&self, id: u64, genre_id: u64) -> DbArtistItem {
        DbArtistItem {
            artist_id: id,
            name: self.artist_name().unwrap_or_else(|| "Unknown Artist".into()),
            genre: genre_id,
        }
    }

    pub fn album_artist(&self, id: u64, genre_id: u64) -> DbArtistItem {
        DbArtistItem {
            artist_id: id,
            name: self
                .data
                .get("ALBUMARTIST")
                .unwrap_or(&TagType::Unknown)
                .str()
                .unwrap_or("Unknown Artist")
                .into(),
            genre: genre_id,
        }
    }

    pub fn album(&self, id: u64, meta_id: u64, artist_id: u64, genre_id: u64) -> DbAlbumItem {
        DbAlbumItem {
            album_id: id,
            title: self.album_title().unwrap_or_else(|| "Unknown Album".into()),
            metadata: meta_id,
            artist: artist_id,
            genre: genre_id,
        }
    }

    pub fn album_meta(&self, id: u64) -> DbMetaItem {
        DbMetaItem {
            meta_id: id,
            plays: self
                .data
                .get("PLAYS")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
            track: self
                .data
                .get("TRACKTOTAL")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
            disc: self
                .data
                .get("DISCTOTAL")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(1),
            duration: 0,
            date: self
                .data
                .get("DATE")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
        }
    }

    pub fn genre(&self, id: u64) -> DbGenreItem {
        DbGenreItem {
            genre_id: id,
            title: self.genre_title().unwrap_or_else(|| "Unknown Genre".into()),
        }
    }
}

#[derive(Clone)]
enum TagType {
    Boolean(bool),
    Flag,
    I64(i64),
    U64(u64),
    Str(String),
    Unknown,
}

impl TagType {
    fn from_symphonia_value(value: &Value) -> Option<Self> {
        match value {
            Value::Binary(_val) => None,
            Value::Boolean(b) => Some(Self::Boolean(*b)),
            Value::Flag => Some(Self::Flag),
            Value::Float(_val) => None,
            Value::SignedInt(i) => Some(Self::I64(*i)),
            Value::String(s) => Some(Self::Str(s.clone())),
            Value::UnsignedInt(u) => Some(Self::U64(*u)),
        }
    }

    fn str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    fn uint(&self) -> Option<u64> {
        match self {
            Self::I64(i) => (*i).try_into().ok(),
            Self::U64(u) => Some(*u),
            Self::Str(s) => s.parse::<u64>().ok(),
            _ => None,
        }
    }
}
