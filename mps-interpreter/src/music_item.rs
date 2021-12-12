use super::lang::db::{DatabaseObj, DbMusicItem};

#[derive(Clone, Debug)]
pub struct MpsMusicItem {
    pub title: String,
    pub filename: String,
}

impl MpsMusicItem {
    pub fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let item = DbMusicItem::map_row(row)?;
        Ok(Self {
            title: item.title,
            filename: item.filename,
        })
    }
}
