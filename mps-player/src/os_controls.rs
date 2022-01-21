#[allow(unused_imports)]
use std::sync::mpsc::{channel, Sender, Receiver};
#[cfg(all(target_os = "linux", feature = "os-controls"))]
use std::thread::JoinHandle;

#[cfg(all(target_os = "linux", feature = "os-controls"))]
use mpris_player::{MprisPlayer, PlaybackStatus, Metadata};

#[cfg(all(target_os = "linux", feature = "os-controls"))]
use mps_interpreter::MpsItem;

//use super::MpsController;
use super::player_wrapper::{ControlAction, PlaybackAction};

/// OS-specific APIs for media controls.
/// Currently only Linux (dbus) is supported.
#[cfg(all(target_os = "linux", feature = "os-controls"))]
pub struct SystemControlWrapper {
    control: Sender<ControlAction>,
    dbus_handle: Option<JoinHandle<()>>, //std::sync::Arc<MprisPlayer>,
    dbus_ctrl: Option<Sender<DbusControl>>,
    playback_event_handler: Option<JoinHandle<()>>,
    playback_event_handler_killer: Option<Sender<()>>,

}

/// OS-specific APIs for media controls.
/// Currently only Linux (dbus) is supported.
#[cfg(not(feature = "os-controls"))]
pub struct SystemControlWrapper {
    #[allow(dead_code)]
    control: Sender<ControlAction>,
    playback_receiver: Option<Receiver<PlaybackAction>>,
}

#[cfg(all(target_os = "linux", feature = "os-controls"))]
enum DbusControl {
    Die,
    SetMetadata(Metadata),
}

#[cfg(all(target_os = "linux", feature = "os-controls"))]
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
            let dbus_conn = MprisPlayer::new("mps".into(), "mps".into(), "ngnius.mps".into());
            //let (msg_tx, msg_rx) = channel();
            // dbus setup
            //self.dbus_conn.set_supported_mime_types(vec![]);
            //self.dbus_conn.set_supported_uri_schemes(vec![]);
            let mut is_playing = true;
            dbus_conn.set_playback_status(PlaybackStatus::Playing);
            dbus_conn.set_can_play(true);
            dbus_conn.set_can_pause(true);
            dbus_conn.set_can_go_next(true);

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

            // poll loop, using my custom mpris lib because original did it wrong
            loop {
                dbus_conn.poll(5);
                match dbus_ctrl.try_recv() {
                    Err(_) => {},
                    Ok(DbusControl::Die) => break,
                    Ok(DbusControl::SetMetadata(meta)) => {
                        dbus_conn.set_metadata(meta);
                    },
                }
            }
        }));
        let (tx, rx) = channel();
        self.playback_event_handler_killer = Some(tx);
        self.playback_event_handler = Some(std::thread::spawn(move || {
            loop {
                if let Ok(_) = rx.try_recv() {
                    break;
                }
                match playback.recv() {
                    Err(_) => break,
                    Ok(PlaybackAction::Exit) => break,
                    Ok(PlaybackAction::Enqueued(item)) => Self::enqueued(item, &dbus_ctrl_tx_clone),
                    Ok(PlaybackAction::Empty) => Self::empty(&dbus_ctrl_tx_clone),
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

    fn enqueued(item: MpsItem, dbus_ctrl: &Sender<DbusControl>) {
        //println!("Got enqueued item {}", &item.title);
        dbus_ctrl.send(DbusControl::SetMetadata(Metadata {
            length: None,
            art_url: None,
            album: item.field("album").and_then(|x| x.to_owned().to_str()),
            album_artist: None, // TODO maybe?
            artist: item.field("artist").and_then(|x| x.to_owned().to_str()).map(|x| vec![x]),
            composer: None,
            disc_number: None,
            genre: item.field("genre").and_then(|x| x.to_owned().to_str()).map(|genre| vec![genre]),
            title: item.field("title").and_then(|x| x.to_owned().to_str()),
            track_number: item.field("track").and_then(|x| x.to_owned().to_i64()).map(|track| track as i32),
            url: item.field("filename").and_then(|x| x.to_owned().to_str()),
        })).unwrap_or(());
    }

    fn empty(dbus_ctrl: &Sender<DbusControl>) {
        dbus_ctrl.send(DbusControl::SetMetadata(Metadata {
            length: None,
            art_url: None,
            album: None,
            album_artist: None, // TODO maybe?
            artist: None,
            composer: None,
            disc_number: None,
            genre: None,
            title: None,
            track_number: None,
            url: None,
        })).unwrap_or(());
    }
}

#[cfg(not(feature = "os-controls"))]
impl SystemControlWrapper {
    pub fn new(control: Sender<ControlAction>) -> Self {
        Self {
            control: control,
            playback_receiver: None
        }
    }

    pub fn init(&mut self, playback: Receiver<PlaybackAction>) {
        self.playback_receiver = Some(playback);
    }

    pub fn exit(self) {}
}
