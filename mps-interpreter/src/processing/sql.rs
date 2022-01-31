use core::fmt::Debug;
use std::collections::{HashMap, HashSet};

use crate::lang::db::*;
use crate::lang::RuntimeMsg;
use crate::MpsItem;

pub type QueryResult = Result<Vec<Result<MpsItem, RuntimeMsg>>, RuntimeMsg>;

/// SQL querying functionality, loosely de-coupled from any specific SQL dialect (excluding raw call)
pub trait MpsDatabaseQuerier: Debug {
    /// raw SQL call, assumed (but not guaranteed) to retrieved music items
    fn raw(&mut self, query: &str) -> QueryResult;

    /// get music, searching by artist name like `query`
    fn artist_like(&mut self, query: &str) -> QueryResult;

    /// get music, searching by album title like `query`
    fn album_like(&mut self, query: &str) -> QueryResult;

    /// get music, searching by song title like `query`
    fn song_like(&mut self, query: &str) -> QueryResult;

    /// get music, searching by genre title like `query`
    fn genre_like(&mut self, query: &str) -> QueryResult;

    /// connect to the SQL database with (optional) settings such as:
    /// `"folder" = "path"` - path to root music directory
    /// `"database" = "uri"` - connection URI for database (for SQLite this is just a filepath)
    /// `"generate" = "true"|"yes"|"false"|"no"` - whether to populate the database using the music directory
    fn init_with_params(&mut self, params: &HashMap<String, String>) -> Result<(), RuntimeMsg>;
}

#[derive(Default, Debug)]
pub struct MpsSQLiteExecutor {
    sqlite_connection: Option<rusqlite::Connection>, // initialized by first SQL statement
}

impl MpsSQLiteExecutor {
    #[inline]
    fn gen_db_maybe(&mut self) -> Result<(), RuntimeMsg> {
        if self.sqlite_connection.is_none() {
            // connection needs to be created
            match generate_default_db() {
                Ok(conn) => {
                    self.sqlite_connection = Some(conn);
                }
                Err(e) => return Err(RuntimeMsg(format!("SQL connection error: {}", e))),
            }
        }
        Ok(())
    }

    fn music_query_single_param(&mut self, query: &str, param: &str) -> QueryResult {
        self.gen_db_maybe()?;
        let conn = self.sqlite_connection.as_mut().unwrap();
        match perform_single_param_query(conn, query, param) {
            Ok(items) => Ok(items
                .into_iter()
                .map(|item| item.map_err(|e| RuntimeMsg(format!("SQL item mapping error: {}", e))))
                .collect()),
            Err(e) => Err(RuntimeMsg(e)),
        }
    }
}

impl MpsDatabaseQuerier for MpsSQLiteExecutor {
    fn raw(&mut self, query: &str) -> QueryResult {
        self.gen_db_maybe()?;
        let conn = self.sqlite_connection.as_mut().unwrap();
        // execute query
        match perform_query(conn, query) {
            Ok(items) => Ok(items
                .into_iter()
                .map(|item| item.map_err(|e| RuntimeMsg(format!("SQL item mapping error: {}", e))))
                .collect()),
            Err(e) => Err(RuntimeMsg(e)),
        }
    }

    fn artist_like(&mut self, query: &str) -> QueryResult {
        let param = &format!("%{}%", query);
        let query_stmt = "SELECT songs.* FROM songs
                JOIN artists ON songs.artist = artists.artist_id
                JOIN metadata ON songs.metadata = metadata.meta_id
            WHERE artists.name like ? ORDER BY songs.album, metadata.track";
        self.music_query_single_param(query_stmt, param)
    }

    fn album_like(&mut self, query: &str) -> QueryResult {
        let param = &format!("%{}%", query);
        let query_stmt = "SELECT songs.* FROM songs
                JOIN albums ON songs.album = artists.album_id
                JOIN metadata ON songs.metadata = metadata.meta_id
            WHERE albums.title like ? ORDER BY songs.album, metadata.track";
        self.music_query_single_param(query_stmt, param)
    }

    fn song_like(&mut self, query: &str) -> QueryResult {
        let param = &format!("%{}%", query);
        let query_stmt = "SELECT songs.* FROM songs
                JOIN metadata ON songs.metadata = metadata.meta_id
            WHERE songs.title like ? ORDER BY songs.album, metadata.track";
        self.music_query_single_param(query_stmt, param)
    }

    fn genre_like(&mut self, query: &str) -> QueryResult {
        let param = &format!("%{}%", query);
        let query_stmt = "SELECT songs.* FROM songs
                JOIN genres ON songs.genre = genres.genre_id
                JOIN metadata ON songs.metadata = metadata.meta_id
            WHERE genres.title like ? ORDER BY songs.album, metadata.track";
        self.music_query_single_param(query_stmt, param)
    }

    fn init_with_params(&mut self, params: &HashMap<String, String>) -> Result<(), RuntimeMsg> {
        // must be executed before connection is created
        if self.sqlite_connection.is_some() {
            Err(RuntimeMsg(
                "Cannot init SQLite connection: Already connected".to_string(),
            ))
        } else {
            // process params
            // init connection
            let mut keys: HashSet<&String> = params.keys().collect();
            let mut settings = SqliteSettings::default();
            for (key, val) in params.iter() {
                let mut match_found = false;
                match key as &str {
                    "folder" | "dir" => {
                        match_found = true;
                        settings.music_path = Some(val.clone());
                    }
                    "database" | "db" => {
                        match_found = true;
                        settings.db_path = Some(val.clone());
                    }
                    "generate" | "gen" => {
                        match_found = true;
                        settings.auto_generate = match val as &str {
                            "true" => Ok(true),
                            "false" => Ok(false),
                            x => Err(RuntimeMsg(format!(
                                "Unrecognised right hand side of param \"{}\" = \"{}\"",
                                key, x
                            ))),
                        }?;
                    }
                    _ => {}
                }
                if match_found {
                    keys.remove(key);
                }
            }
            if !keys.is_empty() {
                // build error msg
                let mut concat_keys = "".to_string();
                let mut first = true;
                for key in keys.drain() {
                    if first {
                        first = false;
                        concat_keys += key;
                    } else {
                        concat_keys += &format!("{}, ", key);
                    }
                }
                return Err(RuntimeMsg(format!(
                    "Unrecognised sql init parameter(s): {}",
                    concat_keys
                )));
            }
            self.sqlite_connection = Some(
                settings
                    .try_into()
                    .map_err(|e| RuntimeMsg(format!("SQL connection error: {}", e)))?,
            );
            Ok(())
        }
    }
}

struct SqliteSettings {
    music_path: Option<String>,
    db_path: Option<String>,
    auto_generate: bool,
}

impl std::default::Default for SqliteSettings {
    fn default() -> Self {
        SqliteSettings {
            music_path: None,
            db_path: None,
            auto_generate: true,
        }
    }
}

impl std::convert::TryInto<rusqlite::Connection> for SqliteSettings {
    type Error = rusqlite::Error;

    fn try_into(self) -> Result<rusqlite::Connection, Self::Error> {
        let music_path = self
            .music_path
            .map(std::path::PathBuf::from)
            .unwrap_or_else(crate::lang::utility::music_folder);
        let sqlite_path = self
            .db_path
            .unwrap_or_else(|| crate::lang::db::DEFAULT_SQLITE_FILEPATH.to_string());
        crate::lang::db::generate_db(music_path, sqlite_path, self.auto_generate)
    }
}

#[inline(always)]
fn build_mps_item(conn: &mut rusqlite::Connection, item: DbMusicItem) -> rusqlite::Result<MpsItem> {
    // query artist
    let mut stmt = conn.prepare_cached("SELECT * from artists WHERE artist_id = ?")?;
    let artist = stmt.query_row([item.artist], DbArtistItem::map_row)?;
    // query album
    let mut stmt = conn.prepare_cached("SELECT * from albums WHERE album_id = ?")?;
    let album = stmt.query_row([item.album], DbAlbumItem::map_row)?;
    // query metadata
    let mut stmt = conn.prepare_cached("SELECT * from metadata WHERE meta_id = ?")?;
    let meta = stmt.query_row([item.metadata], DbMetaItem::map_row)?;
    // query genre
    let mut stmt = conn.prepare_cached("SELECT * from genres WHERE genre_id = ?")?;
    let genre = stmt.query_row([item.genre], DbGenreItem::map_row)?;

    Ok(rows_to_item(item, artist, album, meta, genre))
}

#[inline]
fn perform_query(
    conn: &mut rusqlite::Connection,
    query: &str,
) -> Result<Vec<rusqlite::Result<MpsItem>>, String> {
    let collection: Vec<rusqlite::Result<DbMusicItem>>;
    {
        let mut stmt = conn
            .prepare(query)
            .map_err(|e| format!("SQLite query error: {}", e))?;
        collection = stmt
            .query_map([], DbMusicItem::map_row)
            .map_err(|e| format!("SQLite item mapping error: {}", e))?
            .collect();
    }
    let iter2 = collection.into_iter().map(|item| match item {
        Ok(item) => build_mps_item(conn, item),
        Err(e) => Err(e),
    });
    Ok(iter2.collect())
}

#[inline]
fn perform_single_param_query(
    conn: &mut rusqlite::Connection,
    query: &str,
    param: &str,
) -> Result<Vec<rusqlite::Result<MpsItem>>, String> {
    let collection: Vec<rusqlite::Result<DbMusicItem>>;
    {
        let mut stmt = conn
            .prepare_cached(query)
            .map_err(|e| format!("SQLite query error: {}", e))?;
        collection = stmt
            .query_map([param], DbMusicItem::map_row)
            .map_err(|e| format!("SQLite item mapping error: {}", e))?
            .collect();
    }
    let iter2 = collection.into_iter().map(|item| match item {
        Ok(item) => build_mps_item(conn, item),
        Err(e) => Err(e),
    });
    Ok(iter2.collect())
}

fn rows_to_item(
    music: DbMusicItem,
    artist: DbArtistItem,
    album: DbAlbumItem,
    meta: DbMetaItem,
    genre: DbGenreItem,
) -> MpsItem {
    let mut item = MpsItem::new();
    item
        // music row
        .set_field_chain("title", music.title.into())
        .set_field_chain("filename", music.filename.into())
        // artist row
        .set_field_chain("artist", artist.name.into())
        // album row
        .set_field_chain("album", album.title.into())
        // genre row
        .set_field_chain("genre", genre.title.into())
        // music metadata
        .set_field_chain("track", meta.track.into())
        .set_field_chain("year", meta.date.into());
    item
}
