use super::lang::db::{
    DatabaseObj, DbAlbumItem, DbArtistItem, DbGenreItem, DbMetaItem, DbMusicItem,
};
use super::MpsItem;

#[derive(Clone, Debug)]
pub struct MpsMusicItem {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub filename: String,
    pub genre: Option<String>,
    pub track: Option<u64>,
    pub year: Option<u64>,
}

impl MpsMusicItem {
    pub fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let item = DbMusicItem::map_row(row)?;
        Ok(Self {
            title: item.title,
            artist: None,
            album: None,
            filename: item.filename,
            genre: None,
            track: None,
            year: None,
        })
    }

    pub fn merge(
        music: DbMusicItem,
        artist: DbArtistItem,
        album: DbAlbumItem,
        meta: DbMetaItem,
        genre: DbGenreItem,
    ) -> Self {
        Self {
            title: music.title,
            artist: Some(artist.name),
            album: Some(album.title),
            filename: music.filename,
            genre: Some(genre.title),
            track: Some(meta.track),
            year: Some(meta.date),
        }
    }
}

impl std::convert::From<MpsItem> for MpsMusicItem {
    fn from(mut item: MpsItem) -> Self {
        let default_str = "".to_string();
        Self {
            title: item.remove_field("title").and_then(|x| x.to_str()).unwrap_or(default_str.clone()),
            artist: item.remove_field("artist").and_then(|x| x.to_str()),
            album: item.remove_field("album").and_then(|x| x.to_str()),
            filename: item.remove_field("filename").and_then(|x| x.to_str()).unwrap_or(default_str),
            genre: item.remove_field("genre").and_then(|x| x.to_str()),
            track: item.remove_field("track").and_then(|x| x.to_u64()),
            year: item.remove_field("year").and_then(|x| x.to_u64()),
        }
    }
}

impl std::convert::Into<MpsItem> for MpsMusicItem {
    fn into(self) -> MpsItem {
        let mut result = MpsItem::new();
        result.set_field("title", self.title.into());
        result.set_field("filename", self.filename.into());

        if let Some(artist) = self.artist {result.set_field("artist", artist.into());}
        if let Some(album) = self.album {result.set_field("album", album.into());}
        if let Some(genre) = self.genre {result.set_field("genre", genre.into());}

        if let Some(track) = self.track {result.set_field("track", track.into());}
        if let Some(year) = self.year {result.set_field("year", year.into());}

        result
    }
}
