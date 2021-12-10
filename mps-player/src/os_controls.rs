#[cfg(unix)]
use std::sync::mpsc::{Sender, channel};
#[cfg(unix)]
use std::thread::JoinHandle;

#[cfg(unix)]
use mpris_player::{MprisPlayer, PlaybackStatus};

//use super::MpsController;
use super::player_wrapper::ControlAction;

pub struct SystemControlWrapper {
    control: Sender<ControlAction>,
    #[cfg(target_os = "linux")]
    dbus_handle: Option<JoinHandle<()>>,//std::sync::Arc<MprisPlayer>,
    #[cfg(target_os = "linux")]
    dbus_die: Option<Sender<()>>,
}

#[cfg(target_os = "linux")]
impl SystemControlWrapper {
    pub fn new(control: Sender<ControlAction>) -> Self {
        Self {
            control: control,
            dbus_handle: None,//MprisPlayer::new("mps".into(), "mps".into(), "null".into())
            dbus_die: None,
        }
    }

    pub fn init(&mut self) {
        let (tx, die) = channel();
        self.dbus_die = Some(tx);
        let control_clone1 = self.control.clone();
        self.dbus_handle = Some(std::thread::spawn(move || {
            let dbus_conn = MprisPlayer::new("mps".into(), "mps".into(), "null".into());
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
            dbus_conn.connect_next(
                move || {
                    //println!("Got next signal");
                    control_clone.send(ControlAction::Next{ack: false}).unwrap_or(())
                }
            );

            let control_clone = control_clone1.clone();
            dbus_conn.connect_previous(
                move || control_clone.send(ControlAction::Previous{ack: false}).unwrap_or(())
            );

            let control_clone = control_clone1.clone();
            let dbus_conn_clone = dbus_conn.clone();
            dbus_conn.connect_pause(
                move || {
                    //println!("Got pause signal");
                    dbus_conn_clone.set_playback_status(PlaybackStatus::Paused);
                    control_clone.send(ControlAction::Pause{ack: false}).unwrap_or(());
                }
            );

            let control_clone = control_clone1.clone();
            let dbus_conn_clone = dbus_conn.clone();
            dbus_conn.connect_play(
                move || {
                    //println!("Got play signal");
                    dbus_conn_clone.set_playback_status(PlaybackStatus::Playing);
                    control_clone.send(ControlAction::Play{ack: false}).unwrap_or(())
                }
            );

            let control_clone = control_clone1.clone();
            let dbus_conn_clone = dbus_conn.clone();
            dbus_conn.connect_play_pause(
                move || {
                    //println!("Got play_pause signal (was playing? {})", is_playing);
                    if is_playing {
                        dbus_conn_clone.set_playback_status(PlaybackStatus::Paused);
                        control_clone.send(ControlAction::Pause{ack: false}).unwrap_or(());
                    } else {
                        dbus_conn_clone.set_playback_status(PlaybackStatus::Playing);
                        control_clone.send(ControlAction::Play{ack: false}).unwrap_or(());
                    }
                    is_playing = !is_playing;
                }
            );

            let control_clone = control_clone1.clone();
            dbus_conn.connect_volume(
                move |v| control_clone.send(ControlAction::SetVolume{ack: false, volume: (v * (u32::MAX as f64)) as _}).unwrap_or(())
            );

            // poll loop, using my custom mpris lib because original did it wrong
            loop {
                dbus_conn.poll(5);
                if let Ok(_) = die.try_recv() {
                    break;
                }
            }
        }));
    }

    pub fn exit(self) {
        if let Some(tx) = self.dbus_die {
            tx.send(()).unwrap_or(());
        }
        if let Some(handle) = self.dbus_handle {
            handle.join().unwrap_or(());
        }
    }
}

#[cfg(not(any(target_os = "linux")))]
impl SystemControlWrapper {
    pub fn new(control: Sender<ControlAction>) -> Self {
        Self {
            control: control,
        }
    }

    pub fn init(&mut self) {}

    pub fn exit(self) {}
}
