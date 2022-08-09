#[allow(unused_imports)]
use std::sync::mpsc::{channel, Receiver, Sender};
#[cfg(all(target_os = "linux", feature = "os-controls", feature = "mpris-player"))]
use std::thread::JoinHandle;

#[cfg(all(target_os = "linux", feature = "os-controls", feature = "mpris-player"))]
use mpris_player::{Metadata, MprisPlayer, PlaybackStatus};

#[cfg(all(target_os = "linux", feature = "os-controls", feature = "mpris-player"))]
use muss_interpreter::Item;

//use super::Controller;
use super::player_wrapper::{ControlAction, PlaybackAction};

/// OS-specific APIs for media controls.
/// Currently only Linux (dbus) is supported.
#[cfg(all(target_os = "linux", feature = "os-controls", feature = "mpris-player"))]
pub struct SystemControlWrapper {
    control: Sender<ControlAction>,
    dbus_handle: Option<JoinHandle<()>>, //std::sync::Arc<MprisPlayer>,
    dbus_ctrl: Option<Sender<DbusControl>>,
    playback_event_handler: Option<JoinHandle<()>>,
    playback_event_handler_killer: Option<Sender<()>>,
}

/// OS-specific APIs for media controls.
/// Currently only Linux (dbus) is supported.
#[cfg(any(
    not(feature = "os-controls"),
    not(all(target_os = "linux", feature = "os-controls", feature = "mpris-player"))
))]
pub struct SystemControlWrapper {
    #[allow(dead_code)]
    control: Sender<ControlAction>,
    playback_receiver: Option<Receiver<PlaybackAction>>,
}

#[cfg(all(target_os = "linux", feature = "os-controls", feature = "mpris-player"))]
enum DbusControl {
    Die,
    SetMetadata(Metadata),
    SetPosition(i64),
}

#[cfg(all(target_os = "linux", feature = "os-controls", feature = "mpris-player"))]
impl SystemControlWrapper {
    pub fn new(control: Sender<ControlAction>) -> Self {
        Self {
            control: control,
            dbus_handle: None, //MprisPlayer::new("mps".into(), "mps".into(), "null".into())
            dbus_ctrl: None,
            playback_event_handler: None,
            playback_event_handler_killer: None,
        }
    }

    pub fn init(&mut self, playback: Receiver<PlaybackAction>) {
        let (tx, dbus_ctrl) = channel();
        let dbus_ctrl_tx_clone = tx.clone();
        self.dbus_ctrl = Some(tx);
        let control_clone1 = self.control.clone();
        self.dbus_handle = Some(std::thread::spawn(move || {
            let dbus_conn = MprisPlayer::new("muss".into(), "muss".into(), "ngnius.muss".into());
            //let (msg_tx, msg_rx) = channel();
            // dbus setup
            //self.dbus_conn.set_supported_mime_types(vec![]);
            //self.dbus_conn.set_supported_uri_schemes(vec![]);
            let mut is_playing = true;
            dbus_conn.set_playback_status(PlaybackStatus::Stopped);
            dbus_conn.set_can_play(true);
            dbus_conn.set_can_pause(true);
            dbus_conn.set_can_go_next(true);
            dbus_conn.set_can_seek(false);

            let control_clone = control_clone1.clone();
            dbus_conn.connect_next(move || {
                //println!("Got next signal");
                control_clone
                    .send(ControlAction::Next { ack: false })
                    .unwrap_or(())
            });

            let control_clone = control_clone1.clone();
            dbus_conn.connect_previous(move || {
                control_clone
                    .send(ControlAction::Previous { ack: false })
                    .unwrap_or(())
            });

            let control_clone = control_clone1.clone();
            let dbus_conn_clone = dbus_conn.clone();
            dbus_conn.connect_pause(move || {
                //println!("Got pause signal");
                dbus_conn_clone.set_playback_status(PlaybackStatus::Paused);
                control_clone
                    .send(ControlAction::Pause { ack: false })
                    .unwrap_or(());
            });

            let control_clone = control_clone1.clone();
            let dbus_conn_clone = dbus_conn.clone();
            dbus_conn.connect_play(move || {
                //println!("Got play signal");
                dbus_conn_clone.set_playback_status(PlaybackStatus::Playing);
                control_clone
                    .send(ControlAction::Play { ack: false })
                    .unwrap_or(())
            });

            let control_clone = control_clone1.clone();
            let dbus_conn_clone = dbus_conn.clone();
            dbus_conn.connect_play_pause(move || {
                //println!("Got play_pause signal (was playing? {})", is_playing);
                if is_playing {
                    dbus_conn_clone.set_playback_status(PlaybackStatus::Paused);
                    control_clone
                        .send(ControlAction::Pause { ack: false })
                        .unwrap_or(());
                } else {
                    dbus_conn_clone.set_playback_status(PlaybackStatus::Playing);
                    control_clone
                        .send(ControlAction::Play { ack: false })
                        .unwrap_or(());
                }
                is_playing = !is_playing;
            });

            let control_clone = control_clone1.clone();
            dbus_conn.connect_volume(move |v| {
                control_clone
                    .send(ControlAction::SetVolume {
                        ack: false,
                        volume: (v * (u32::MAX as f64)) as _,
                    })
                    .unwrap_or(())
            });

            dbus_conn.set_playback_status(PlaybackStatus::Playing);

            // poll loop, using my custom mpris lib because original did it wrong
            loop {
                dbus_conn.poll(5);
                match dbus_ctrl.try_recv() {
                    Err(_) => {}
                    Ok(DbusControl::Die) => break,
                    Ok(DbusControl::SetMetadata(meta)) => {
                        dbus_conn.set_metadata(meta);
                    },
                    Ok(DbusControl::SetPosition(pos)) => {
                        dbus_conn.set_position(pos);
                    }
                }
            }
        }));
        let (tx, rx) = channel();
        self.playback_event_handler_killer = Some(tx);
        self.playback_event_handler = Some(std::thread::spawn(move || {
            let mut playback_time = 0;
            let mut duration_cache = None;
            loop {
                if let Ok(_) = rx.try_recv() {
                    break;
                }
                match playback.recv() {
                    Err(_) => break,
                    Ok(PlaybackAction::Exit) => break,
                    Ok(PlaybackAction::Enqueued(item)) => {
                        playback_time = 0;
                        duration_cache = None;
                        Self::enqueued(item, &dbus_ctrl_tx_clone);
                    },
                    Ok(PlaybackAction::Empty) => Self::empty(&dbus_ctrl_tx_clone),
                    Ok(PlaybackAction::Time(item, duration)) => {
                        duration_cache = Some(duration);
                        Self::time(item, duration, &dbus_ctrl_tx_clone);
                    },
                    Ok(PlaybackAction::UpdateTick(item)) => {
                        Self::time_update(item, playback_time, &duration_cache, &dbus_ctrl_tx_clone);
                        playback_time += 1;
                    },
                }
            }
        }));
    }

    pub fn exit(self) {
        // exit dbus thread
        if let Some(tx) = self.dbus_ctrl {
            tx.send(DbusControl::Die).unwrap_or(());
        }
        if let Some(handle) = self.dbus_handle {
            handle.join().unwrap_or(());
        }
        // exit playback event thread
        if let Some(tx) = self.playback_event_handler_killer {
            tx.send(()).unwrap_or(());
        }
        if let Some(handle) = self.playback_event_handler {
            handle.join().unwrap_or(());
        }
    }

    fn build_metadata(item: Item) -> Metadata {
        let file_uri = item.field("filename").and_then(|x| x.to_owned().to_str());
        Metadata {
            length: None,
            art_url: None, //file_uri.clone() TODO do this without having to rip the art image from the file like Elisa
            album: item.field("album").and_then(|x| x.to_owned().to_str()),
            album_artist: item
                .field("albumartist")
                .map(
                    |x| x.to_owned()
                        .to_str()
                        .map(|x2| vec![x2])
                ).flatten(),
            artist: item
                .field("artist")
                .and_then(|x| x.to_owned().to_str())
                .map(|x| x.split(",").map(|s| s.trim().to_owned()).collect()),
            composer: None,
            disc_number: None,
            genre: item
                .field("genre")
                .and_then(|x| x.to_owned().to_str())
                .map(|genre| vec![genre]),
            title: item.field("title").and_then(|x| x.to_owned().to_str()),
            track_number: item
                .field("track")
                .and_then(|x| x.to_owned().to_i64())
                .map(|track| track as i32),
            url: file_uri,
        }
    }

    fn enqueued(item: Item, dbus_ctrl: &Sender<DbusControl>) {
        //println!("Got enqueued item {}", &item.title);
        dbus_ctrl
            .send(DbusControl::SetMetadata(Self::build_metadata(item)))
            .unwrap_or(());
    }

    fn empty(dbus_ctrl: &Sender<DbusControl>) {
        dbus_ctrl
            .send(DbusControl::SetMetadata(Metadata {
                length: None,
                art_url: None,
                album: None,
                album_artist: None,
                artist: None,
                composer: None,
                disc_number: None,
                genre: None,
                title: None,
                track_number: None,
                url: None,
            }))
            .unwrap_or(());
    }

    fn time(item: Item, duration: std::time::Duration, dbus_ctrl: &Sender<DbusControl>) {
        let mut meta = Self::build_metadata(item);
        meta.length = Some(duration.as_secs_f64().round() as i64 * 1_000_000);
        dbus_ctrl
            .send(DbusControl::SetMetadata(meta))
            .unwrap_or(());
    }

    fn time_update(_item: Item, new_time: i64, duration: &Option<std::time::Duration>, dbus_ctrl: &Sender<DbusControl>) {
        //println!("Position update tick");
        if duration.is_some() {
            /*let mut meta = Self::build_metadata(item);
            meta.length = Some(new_time + 1);
            dbus_ctrl
                .send(DbusControl::SetMetadata(meta))
                .unwrap_or(());*/
            dbus_ctrl
                .send(DbusControl::SetPosition(new_time * 1_000_000))
                .unwrap_or(());
        }
    }
}

#[cfg(any(
    not(feature = "os-controls"),
    not(all(target_os = "linux", feature = "os-controls", feature = "mpris-player"))
))]
impl SystemControlWrapper {
    pub fn new(control: Sender<ControlAction>) -> Self {
        Self {
            control: control,
            playback_receiver: None,
        }
    }

    pub fn init(&mut self, playback: Receiver<PlaybackAction>) {
        self.playback_receiver = Some(playback);
    }

    pub fn exit(self) {}
}
