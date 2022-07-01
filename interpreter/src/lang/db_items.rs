use std::path::Path;

pub const DEFAULT_SQLITE_FILEPATH: &str = "metadata.sqlite";

pub trait DatabaseObj: Sized {
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self>;

    fn to_params(&self) -> Vec<&'_ dyn rusqlite::ToSql>;

    fn id(&self) -> u64;
}

pub fn generate_default_db() -> rusqlite::Result<rusqlite::Connection> {
    generate_db(
        super::utility::music_folder(),
        DEFAULT_SQLITE_FILEPATH,
        true,
    )
}

pub fn generate_db<P1: AsRef<Path>, P2: AsRef<Path>>(
    music_path: P1,
    sqlite_path: P2,
    generate: bool,
) -> rusqlite::Result<rusqlite::Connection> {
    #[allow(unused_variables)]
    let music_path = music_path.as_ref();
    let sqlite_path = sqlite_path.as_ref();
    let db_exists = std::path::Path::new(sqlite_path).exists();
    #[cfg(not(feature = "music_library"))]
    let conn = rusqlite::Connection::open(sqlite_path)?;
    #[cfg(feature = "music_library")]
    let mut conn = rusqlite::Connection::open(sqlite_path)?;
    // skip db building if SQLite file already exists
    // TODO do a more exhaustive db check to make sure it's actually the correct file and database structure
    #[cfg(not(feature = "music_library"))]
    if db_exists && !generate {
        return Ok(conn);
    }
    // build db tables
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS songs (
            song_id INTEGER NOT NULL PRIMARY KEY,
            title TEXT NOT NULL,
            artist INTEGER NOT NULL,
            album INTEGER,
            filename TEXT,
            metadata INTEGER NOT NULL,
            genre INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS artists (
            artist_id INTEGER NOT NULL PRIMARY KEY,
            name TEXT NOT NULL,
            genre INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS albums (
            album_id INTEGER NOT NULL PRIMARY KEY,
            title TEXT,
            metadata INTEGER NOT NULL,
            artist INTEGER NOT NULL,
            genre INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS metadata (
            meta_id INTEGER NOT NULL PRIMARY KEY,
            plays INTEGER NOT NULL DEFAULT 0,
            track INTEGER NOT NULL DEFAULT 1,
            disc INTEGER NOT NULL DEFAULT 1,
            duration INTEGER,
            date INTEGER
        );
        CREATE TABLE IF NOT EXISTS genres (
            genre_id INTEGER NOT NULL PRIMARY KEY,
            title TEXT
        );
        COMMIT;",
    )?;
    // generate data and store in db
    #[cfg(feature = "music_library")]
    if generate {
        let mut lib = crate::music::Library::new();
        if db_exists {
            crate::music::build_library_from_sqlite(&conn, &mut lib)?;
        }
        lib.clear_modified();
        match crate::music::build_library_from_files(&music_path, &mut lib) {
            Ok(_) => {
                if lib.is_modified() {
                    let transaction = conn.transaction()?;
                    {
                        let mut song_insert = transaction.prepare(
                            "INSERT OR REPLACE INTO songs (
                        song_id,
                        title,
                        artist,
                        album,
                        filename,
                        metadata,
                        genre
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                        )?;
                        for song in lib.all_songs() {
                            song_insert.execute(song.to_params().as_slice())?;
                        }

                        let mut metadata_insert = transaction.prepare(
                            "INSERT OR REPLACE INTO metadata (
                        meta_id,
                        plays,
                        track,
                        disc,
                        duration,
                        date
                    ) VALUES (?, ?, ?, ?, ?, ?)",
                        )?;
                        for meta in lib.all_metadata() {
                            metadata_insert.execute(meta.to_params().as_slice())?;
                        }

                        let mut artist_insert = transaction.prepare(
                            "INSERT OR REPLACE INTO artists (
                        artist_id,
                        name,
                        genre
                    ) VALUES (?, ?, ?)",
                        )?;
                        for artist in lib.all_artists() {
                            artist_insert.execute(artist.to_params().as_slice())?;
                        }

                        let mut album_insert = transaction.prepare(
                            "INSERT OR REPLACE INTO albums (
                        album_id,
                        title,
                        metadata,
                        artist,
                        genre
                    ) VALUES (?, ?, ?, ?, ?)",
                        )?;
                        for album in lib.all_albums() {
                            album_insert.execute(album.to_params().as_slice())?;
                        }

                        let mut genre_insert = transaction.prepare(
                            "INSERT OR REPLACE INTO genres (
                        genre_id,
                        title
                    ) VALUES (?, ?)",
                        )?;
                        for genre in lib.all_genres() {
                            genre_insert.execute(genre.to_params().as_slice())?;
                        }
                    }
                    transaction.commit()?;
                }
            }
            Err(e) => println!("Unable to load music from {}: {}", music_path.display(), e),
        }
    }
    Ok(conn)
}

#[derive(Clone, Debug)]
pub struct DbMusicItem {
    pub song_id: u64,
    pub title: String,
    pub artist: u64,
    pub album: Option<u64>,
    pub filename: String,
    pub metadata: u64,
    pub genre: u64,
}

impl DatabaseObj for DbMusicItem {
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            song_id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            album: row.get(3)?,
            filename: row.get(4)?,
            metadata: row.get(5)?,
            genre: row.get(6)?,
        })
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.song_id,
            &self.title,
            &self.artist,
            &self.album,
            &self.filename,
            &self.metadata,
            &self.genre,
        ]
    }

    fn id(&self) -> u64 {
        self.song_id
    }
}

#[derive(Clone, Debug)]
pub struct DbMetaItem {
    pub meta_id: u64,
    pub plays: u64,
    pub track: u64,
    pub disc: u64,
    pub duration: u64, // seconds
    pub date: u64,     // year
}

impl DatabaseObj for DbMetaItem {
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            meta_id: row.get(0)?,
            plays: row.get(1)?,
            track: row.get(2)?,
            disc: row.get(3)?,
            duration: row.get(4)?,
            date: row.get(5)?,
        })
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.meta_id,
            &self.plays,
            &self.track,
            &self.disc,
            &self.duration,
            &self.date,
        ]
    }

    fn id(&self) -> u64 {
        self.meta_id
    }
}

#[derive(Clone, Debug)]
pub struct DbArtistItem {
    pub artist_id: u64,
    pub name: String,
    pub genre: u64,
}

impl DatabaseObj for DbArtistItem {
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            artist_id: row.get(0)?,
            name: row.get(1)?,
            genre: row.get(2)?,
        })
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.artist_id, &self.name, &self.genre]
    }

    fn id(&self) -> u64 {
        self.artist_id
    }
}

#[derive(Clone, Debug)]
pub struct DbAlbumItem {
    pub album_id: u64,
    pub title: String,
    pub metadata: u64,
    pub artist: u64,
    pub genre: u64,
}

impl DatabaseObj for DbAlbumItem {
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            album_id: row.get(0)?,
            title: row.get(1)?,
            metadata: row.get(2)?,
            artist: row.get(3)?,
            genre: row.get(4)?,
        })
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.album_id,
            &self.title,
            &self.metadata,
            &self.artist,
            &self.genre,
        ]
    }

    fn id(&self) -> u64 {
        self.album_id
    }
}

#[derive(Clone, Debug)]
pub struct DbGenreItem {
    pub genre_id: u64,
    pub title: String,
}

impl DatabaseObj for DbGenreItem {
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            genre_id: row.get(0)?,
            title: row.get(1)?,
        })
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.genre_id, &self.title]
    }

    fn id(&self) -> u64 {
        self.genre_id
    }
}
