use std::fs;
use std::io;

use rodio::{decoder::Decoder, OutputStream, OutputStreamHandle, Sink};

use m3u8_rs::{MediaPlaylist, MediaSegment};

#[cfg(feature = "mpd")]
use mpd::{Client, Song, error};

use super::uri::Uri;

use muss_interpreter::{tokens::TokenReader, Interpreter, Item};

//use super::PlaybackError;
use super::PlayerError;
use super::UriError;

/// Playback functionality for a script.
/// This takes the output of the runner and plays or saves it.
pub struct Player<'a, T: TokenReader + 'a> {
    runner: Interpreter<'a, T>,
    sink: Sink,
    #[allow(dead_code)]
    output_stream: OutputStream, // this is required for playback, so it must live as long as this struct instance
    output_handle: OutputStreamHandle,
    #[cfg(feature = "mpd")]
    mpd_connection: Option<Client<std::net::TcpStream>>,
}

impl<'a, T: TokenReader + 'a> Player<'a, T> {
    pub fn new(runner: Interpreter<'a, T>) -> Result<Self, PlayerError> {
        let (stream, output_handle) =
            OutputStream::try_default().map_err(PlayerError::from_err_playback)?;
        Ok(Self {
            runner: runner,
            sink: Sink::try_new(&output_handle).map_err(PlayerError::from_err_playback)?,
            output_stream: stream,
            output_handle: output_handle,
            #[cfg(feature = "mpd")]
            mpd_connection: None,
        })
    }

    #[cfg(feature = "mpd")]
    pub fn connect_mpd(&mut self, addr: std::net::SocketAddr) -> Result<(), PlayerError> {
        self.mpd_connection = Some(Client::connect(addr).map_err(PlayerError::from_err_mpd)?);
        Ok(())
    }

    #[cfg(feature = "mpd")]
    pub fn set_mpd(&mut self, client: Client<std::net::TcpStream>) {
        self.mpd_connection = Some(client);
    }

    pub fn play_all(&mut self) -> Result<(), PlayerError> {
        while let Some(item) = self.runner.next() {
            self.sink.sleep_until_end();
            match item {
                Ok(music) => {
                    if let Some(filename) =
                        music.field("filename").and_then(|x| x.to_owned().to_str())
                    {
                        self.append_source(&filename)?;
                        Ok(())
                    } else {
                        Err(PlayerError::from_err_playback(
                            "Field `filename` does not exist on item",
                        ))
                    }
                }
                Err(e) => Err(PlayerError::from_err_playback(e)),
            }?;
        }
        self.sink.sleep_until_end();
        Ok(())
    }

    pub fn enqueue_all(&mut self) -> Result<Vec<Item>, PlayerError> {
        let mut enqueued = Vec::new();
        while let Some(item) = self.runner.next() {
            match item {
                Ok(music) => {
                    enqueued.push(music.clone());
                    if let Some(filename) =
                        music.field("filename").and_then(|x| x.to_owned().to_str())
                    {
                        self.append_source(&filename)?;
                        Ok(())
                    } else {
                        Err(PlayerError::from_err_playback(
                            "Field `filename` does not exist on item",
                        ))
                    }
                }
                Err(e) => Err(PlayerError::from_err_playback(e)),
            }?;
        }
        Ok(enqueued)
    }

    pub fn enqueue(&mut self, count: usize) -> Result<Vec<Item>, PlayerError> {
        let mut items_left = count;
        let mut enqueued = Vec::with_capacity(count);
        if items_left == 0 {
            return Ok(enqueued);
        }
        while let Some(item) = self.runner.next() {
            match item {
                Ok(music) => {
                    if let Some(filename) =
                        music.field("filename").and_then(|x| x.to_owned().to_str())
                    {
                        enqueued.push(music.clone());
                        self.append_source(&filename)?;
                        items_left -= 1;
                        Ok(())
                    } else {
                        Err(PlayerError::from_err_playback(
                            "Field `filename` does not exist on item",
                        ))
                    }
                }
                Err(e) => Err(PlayerError::from_err_playback(e)),
            }?;
            if items_left == 0 {
                break;
            }
        }
        //println!("Enqueued {} items", count - items_left);
        Ok(enqueued)
    }

    pub fn resume(&self) {
        self.sink.play()
    }

    pub fn pause(&self) {
        self.sink.pause()
    }

    pub fn stop(&self) {
        self.sink.stop()
    }

    pub fn sleep_until_end(&self) {
        self.sink.sleep_until_end()
    }

    pub fn queue_len(&self) -> usize {
        self.sink.len()
    }

    pub fn queue_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn save_m3u8<W: io::Write>(&mut self, w: &mut W) -> Result<(), PlayerError> {
        let mut playlist = MediaPlaylist {
            version: 6,
            ..Default::default()
        };
        // generate
        for item in &mut self.runner {
            match item {
                Ok(music) => {
                    if let Some(filename) =
                        music_filename(&music)
                    {
                        //println!("Adding file `{}` to playlist", filename);
                        playlist.segments.push(MediaSegment {
                            uri: filename,
                            title: music_title(&music),
                            ..Default::default()
                        });
                        Ok(())
                    } else {
                        Err(PlayerError::from_err_playback(
                            "Field `filename` does not exist on item",
                        ))
                    }
                }
                Err(e) => Err(PlayerError::from_err_playback(e)),
            }?;
        }
        playlist.write_to(w).map_err(PlayerError::from_err_playback)
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn new_sink(&mut self) -> Result<(), PlayerError> {
        let is_paused = self.sink.is_paused();
        let volume = self.sink.volume();

        self.stop();
        self.sink = Sink::try_new(&self.output_handle).map_err(PlayerError::from_err_playback)?;

        if is_paused {
            self.sink.pause();
        }
        self.sink.set_volume(volume);
        Ok(())
    }

    fn append_source(&mut self, filename: &str) -> Result<(), PlayerError> {
        let uri = Uri::new(filename);
        match uri.scheme() {
            Some(s) => match &s.to_lowercase() as &str {
                "file:" => {
                    let file = fs::File::open(uri.path()).map_err(PlayerError::from_err_playback)?;
                    let stream = io::BufReader::new(file);
                    let source = Decoder::new(stream).map_err(PlayerError::from_err_playback)?;
                    self.sink.append(source);
                    Ok(())
                },
                #[cfg(feature = "mpd")]
                "mpd:" => {
                    if let Some(mpd_client) = &mut self.mpd_connection {
                        //println!("Pushing {} into MPD queue", uri.path());
                        let song = Song {
                            file: uri.path().to_owned(),
                            ..Default::default()
                        };
                        mpd_client.push(song).map_err(PlayerError::from_err_playback)?;
                        Ok(())
                    } else {
                        Err(PlayerError::from_err_playback("Cannot play MPD song: no MPD client connected"))
                    }
                },
                scheme => Err(UriError::Unsupported(scheme.to_owned()).into())
            },
            None => {
                //default
                // NOTE: Default rodio::Decoder hangs here when decoding large files, but symphonia does not
                let file = fs::File::open(uri.path()).map_err(PlayerError::from_err_playback)?;
                let stream = io::BufReader::new(file);
                let source = Decoder::new(stream).map_err(PlayerError::from_err_playback)?;
                self.sink.append(source);
                Ok(())
            }
        }
    }
}

#[cfg(feature = "mpd")]
pub fn mpd_connection(addr: std::net::SocketAddr) -> error::Result<Client<std::net::TcpStream>> {
    Client::connect(addr)
}

#[inline]
fn music_title(item: &Item) -> Option<String> {
    item.field("title").and_then(|x| x.to_owned().to_str())
}

#[inline]
fn music_filename(item: &Item) -> Option<String> {
    if let Some(filename) = item.field("filename") {
        if let Ok(cwd) = std::env::current_dir() {
            let path: &std::path::Path = &cwd;
            Some(filename.as_str().replace(path.to_str().unwrap_or(""), "./"))
        } else {
            Some(filename.to_string())
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mps_interpreter::Interpreter;
    use std::io;

    #[allow(dead_code)]
    //#[test]
    fn play_cursor() -> Result<(), PlayerError> {
        let cursor = io::Cursor::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
        let runner = Interpreter::with_stream(cursor);
        let mut player = Player::new(runner)?;
        player.play_all()
    }

    #[test]
    fn playlist() -> Result<(), PlayerError> {
        let cursor = io::Cursor::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
        let runner = Interpreter::with_stream(cursor);
        let mut player = Player::new(runner)?;

        let output_file = std::fs::File::create("playlist.m3u8").unwrap();
        let mut buffer = std::io::BufWriter::new(output_file);
        player.save_m3u8(&mut buffer)
    }
}
