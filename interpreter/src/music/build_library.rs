use std::path::Path;

use super::Library;
use crate::lang::db::*;

pub fn build_library_from_files<P: AsRef<Path>>(
    path: P,
    lib: &mut Library,
) -> std::io::Result<()> {
    //let mut result = Library::new();
    lib.read_path(path, 10)?;
    Ok(())
}

pub fn build_library_from_sqlite(
    conn: &rusqlite::Connection,
    lib: &mut Library,
) -> rusqlite::Result<()> {
    // build songs
    for song in conn
        .prepare("SELECT * from songs")?
        .query_map([], DbMusicItem::map_row)?
    {
        lib.add_song(song?);
    }
    // build metadata
    for meta in conn
        .prepare("SELECT * from metadata")?
        .query_map([], DbMetaItem::map_row)?
    {
        lib.add_metadata(meta?);
    }
    // build artists
    for artist in conn
        .prepare("SELECT * from artists")?
        .query_map([], DbArtistItem::map_row)?
    {
        lib.add_artist(artist?);
    }
    // build albums
    for album in conn
        .prepare("SELECT * from albums")?
        .query_map([], DbAlbumItem::map_row)?
    {
        lib.add_album(album?);
    }
    // build genres
    for genre in conn
        .prepare("SELECT * from genres")?
        .query_map([], DbGenreItem::map_row)?
    {
        lib.add_genre(genre?);
    }
    Ok(())
}
