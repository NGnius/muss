use std::fs;
use std::io;

use rodio::{decoder::Decoder, OutputStream, OutputStreamHandle, Sink};

use m3u8_rs::{MediaPlaylist, MediaSegment};

use mps_interpreter::{tokens::MpsTokenReader, MpsItem, MpsRunner};

use super::PlaybackError;

/// Playback functionality for a script.
/// This takes the output of the runner and plays or saves it.
pub struct MpsPlayer<T: MpsTokenReader> {
    runner: MpsRunner<T>,
    sink: Sink,
    #[allow(dead_code)]
    output_stream: OutputStream, // this is required for playback, so it must live as long as this struct instance
    output_handle: OutputStreamHandle,
}

impl<T: MpsTokenReader> MpsPlayer<T> {
    pub fn new(runner: MpsRunner<T>) -> Result<Self, PlaybackError> {
        let (stream, output_handle) =
            OutputStream::try_default().map_err(PlaybackError::from_err)?;
        Ok(Self {
            runner: runner,
            sink: Sink::try_new(&output_handle).map_err(PlaybackError::from_err)?,
            output_stream: stream,
            output_handle: output_handle,
        })
    }

    pub fn play_all(&mut self) -> Result<(), PlaybackError> {
        for item in &mut self.runner {
            self.sink.sleep_until_end();
            match item {
                Ok(music) => {
                    if let Some(filename) =
                        music.field("filename").and_then(|x| x.to_owned().to_str())
                    {
                        // NOTE: Default rodio::Decoder hangs here when decoding large files, but symphonia does not
                        let file = fs::File::open(filename).map_err(PlaybackError::from_err)?;
                        let stream = io::BufReader::new(file);
                        let source = Decoder::new(stream).map_err(PlaybackError::from_err)?;
                        self.sink.append(source);
                        Ok(())
                    } else {
                        Err(PlaybackError::from_err(
                            "Field `filename` does not exist on item",
                        ))
                    }
                },
                Err(e) => Err(PlaybackError::from_err(e)),
            }?;
        }
        self.sink.sleep_until_end();
        Ok(())
    }

    pub fn enqueue_all(&mut self) -> Result<Vec<MpsItem>, PlaybackError> {
        let mut enqueued = Vec::new();
        for item in &mut self.runner {
            match item {
                Ok(music) => {
                    enqueued.push(music.clone());
                    if let Some(filename) =
                        music.field("filename").and_then(|x| x.to_owned().to_str())
                    {
                        // NOTE: Default rodio::Decoder hangs here when decoding large files, but symphonia does not
                        let file = fs::File::open(filename).map_err(PlaybackError::from_err)?;
                        let stream = io::BufReader::new(file);
                        let source = Decoder::new(stream).map_err(PlaybackError::from_err)?;
                        self.sink.append(source);
                        Ok(())
                    } else {
                        Err(PlaybackError::from_err(
                            "Field `filename` does not exist on item",
                        ))
                    }
                }
                Err(e) => Err(PlaybackError::from_err(e)),
            }?;
        }
        Ok(enqueued)
    }

    pub fn enqueue(&mut self, count: usize) -> Result<Vec<MpsItem>, PlaybackError> {
        let mut items_left = count;
        let mut enqueued = Vec::with_capacity(count);
        if items_left == 0 {
            return Ok(enqueued);
        }
        for item in &mut self.runner {
            match item {
                Ok(music) => {
                    if let Some(filename) =
                        music.field("filename").and_then(|x| x.to_owned().to_str())
                    {
                        enqueued.push(music.clone());
                        // NOTE: Default rodio::Decoder hangs here when decoding large files, but symphonia does not
                        let file = fs::File::open(filename).map_err(PlaybackError::from_err)?;
                        let stream = io::BufReader::new(file);
                        let source = Decoder::new(stream).map_err(PlaybackError::from_err)?;
                        self.sink.append(source);
                        items_left -= 1;
                        Ok(())
                    } else {
                        Err(PlaybackError::from_err(
                            "Field `filename` does not exist on item",
                        ))
                    }
                }
                Err(e) => Err(PlaybackError::from_err(e)),
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

    pub fn save_m3u8<W: io::Write>(&mut self, w: &mut W) -> Result<(), PlaybackError> {
        let mut playlist = MediaPlaylist {
            version: 6,
            ..Default::default()
        };
        // generate
        for item in &mut self.runner {
            match item {
                Ok(music) => {
                    if let Some(filename) =
                        music.field("filename").and_then(|x| x.to_owned().to_str())
                    {
                        playlist.segments.push(MediaSegment {
                            uri: filename,
                            title: music.field("title").and_then(|x| x.to_owned().to_str()),
                            ..Default::default()
                        });
                        Ok(())
                    } else {
                        Err(PlaybackError::from_err(
                            "Field `filename` does not exist on item",
                        ))
                    }
                }
                Err(e) => Err(PlaybackError::from_err(e)),
            }?;
        }
        playlist.write_to(w).map_err(PlaybackError::from_err)
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn new_sink(&mut self) -> Result<(), PlaybackError> {
        let is_paused = self.sink.is_paused();
        let volume = self.sink.volume();

        self.stop();
        self.sink = Sink::try_new(&self.output_handle).map_err(PlaybackError::from_err)?;

        if is_paused {
            self.sink.pause();
        }
        self.sink.set_volume(volume);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mps_interpreter::MpsRunner;
    use std::io;

    #[allow(dead_code)]
    //#[test]
    fn play_cursor() -> Result<(), PlaybackError> {
        let cursor = io::Cursor::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
        let runner = MpsRunner::with_stream(cursor);
        let mut player = MpsPlayer::new(runner)?;
        player.play_all()
    }

    #[test]
    fn playlist() -> Result<(), PlaybackError> {
        let cursor = io::Cursor::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
        let runner = MpsRunner::with_stream(cursor);
        let mut player = MpsPlayer::new(runner)?;

        let output_file = std::fs::File::create("playlist.m3u8").unwrap();
        let mut buffer = std::io::BufWriter::new(output_file);
        player.save_m3u8(&mut buffer)
    }
}
