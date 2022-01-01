use super::lang::db::{
    DatabaseObj, DbAlbumItem, DbArtistItem, DbGenreItem, DbMetaItem, DbMusicItem,
};

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
