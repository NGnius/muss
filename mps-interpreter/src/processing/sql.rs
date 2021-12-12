use core::fmt::Debug;

use crate::lang::db::*;
use crate::lang::{MpsOp, RuntimeError};
use crate::MpsMusicItem;

pub type QueryResult = Result<Vec<Result<MpsMusicItem, RuntimeError>>, RuntimeError>;
pub type QueryOp = dyn FnMut() -> Box<dyn MpsOp>;

pub trait MpsDatabaseQuerier: Debug {
    fn raw(&mut self, query: &str, op: &mut QueryOp) -> QueryResult;

    fn artist_like(&mut self, query: &str, op: &mut QueryOp) -> QueryResult;

    fn album_like(&mut self, query: &str, op: &mut QueryOp) -> QueryResult;

    fn song_like(&mut self, query: &str, op: &mut QueryOp) -> QueryResult;

    fn genre_like(&mut self, query: &str, op: &mut QueryOp) -> QueryResult;
}

#[derive(Default, Debug)]
pub struct MpsSQLiteExecutor {
    sqlite_connection: Option<rusqlite::Connection>, // initialized by first SQL statement
}

impl MpsSQLiteExecutor {
    #[inline]
    fn gen_db_maybe(&mut self, op: &mut QueryOp) -> Result<(), RuntimeError> {
        if let None = self.sqlite_connection {
            // connection needs to be created
            match generate_default_db() {
                Ok(conn) => {
                    self.sqlite_connection = Some(conn);
                }
                Err(e) => {
                    return Err(RuntimeError {
                        line: 0,
                        op: op(),
                        msg: format!("SQL connection error: {}", e).into(),
                    })
                }
            }
        }
        Ok(())
    }

    fn music_query_single_param(
        &mut self,
        query: &str,
        param: &str,
        op: &mut QueryOp,
    ) -> QueryResult {
        self.gen_db_maybe(op)?;
        let conn = self.sqlite_connection.as_mut().unwrap();
        match perform_single_param_query(conn, query, param) {
            Ok(items) => Ok(items
                .into_iter()
                .map(|item| {
                    item.map_err(|e| RuntimeError {
                        line: 0,
                        op: op(),
                        msg: format!("SQL item mapping error: {}", e).into(),
                    })
                })
                .collect()),
            Err(e) => Err(RuntimeError {
                line: 0,
                op: op(),
                msg: e,
            }),
        }
    }
}

impl MpsDatabaseQuerier for MpsSQLiteExecutor {
    fn raw(&mut self, query: &str, op: &mut QueryOp) -> QueryResult {
        self.gen_db_maybe(op)?;
        let conn = self.sqlite_connection.as_mut().unwrap();
        // execute query
        match perform_query(conn, query) {
            Ok(items) => Ok(items
                .into_iter()
                .map(|item| {
                    item.map_err(|e| RuntimeError {
                        line: 0,
                        op: op(),
                        msg: format!("SQL item mapping error: {}", e).into(),
                    })
                })
                .collect()),
            Err(e) => Err(RuntimeError {
                line: 0,
                op: op(),
                msg: e,
            }),
        }
    }

    fn artist_like(&mut self, query: &str, op: &mut QueryOp) -> QueryResult {
        let param = &format!("%{}%", query);
        let query_stmt =
            "SELECT songs.* FROM songs
                JOIN artists ON songs.artist = artists.artist_id
                JOIN metadata ON songs.metadata = metadata.meta_id
            WHERE artists.name like ? ORDER BY songs.album, metadata.track";
        self.music_query_single_param(query_stmt, param, op)
    }

    fn album_like(&mut self, query: &str, op: &mut QueryOp) -> QueryResult {
        let param = &format!("%{}%", query);
        let query_stmt =
            "SELECT songs.* FROM songs
                JOIN albums ON songs.album = artists.album_id
                JOIN metadata ON songs.metadata = metadata.meta_id
            WHERE albums.title like ? ORDER BY songs.album, metadata.track";
        self.music_query_single_param(query_stmt, param, op)
    }

    fn song_like(&mut self, query: &str, op: &mut QueryOp) -> QueryResult {
        let param = &format!("%{}%", query);
        let query_stmt =
            "SELECT songs.* FROM songs
                JOIN metadata ON songs.metadata = metadata.meta_id
            WHERE songs.title like ? ORDER BY songs.album, metadata.track";
        self.music_query_single_param(query_stmt, param, op)
    }

    fn genre_like(&mut self, query: &str, op: &mut QueryOp) -> QueryResult {
        let param = &format!("%{}%", query);
        let query_stmt =
            "SELECT songs.* FROM songs
                JOIN genres ON songs.genre = genres.genre_id
                JOIN metadata ON songs.metadata = metadata.meta_id
            WHERE genres.title like ? ORDER BY songs.album, metadata.track";
        self.music_query_single_param(query_stmt, param, op)
    }
}

#[inline]
fn perform_query(
    conn: &mut rusqlite::Connection,
    query: &str,
) -> Result<Vec<rusqlite::Result<MpsMusicItem>>, String> {
    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("SQLite query error: {}", e))?;
    let iter = stmt
        .query_map([], MpsMusicItem::map_row)
        .map_err(|e| format!("SQLite item mapping error: {}", e))?;
    Ok(iter.collect())
}

#[inline]
fn perform_single_param_query(
    conn: &mut rusqlite::Connection,
    query: &str,
    param: &str,
) -> Result<Vec<rusqlite::Result<MpsMusicItem>>, String> {
    let mut stmt = conn
        .prepare_cached(query)
        .map_err(|e| format!("SQLite query error: {}", e))?;
    let iter = stmt
        .query_map([param], MpsMusicItem::map_row)
        .map_err(|e| format!("SQLite item mapping error: {}", e))?;
    Ok(iter.collect())
}
