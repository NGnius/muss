use std::sync::mpsc::{Sender, Receiver};
use std::{thread, thread::JoinHandle};

use mps_interpreter::tokens::MpsTokenReader;

use super::MpsPlayer;
use super::PlaybackError;

pub struct MpsPlayerServer<T: MpsTokenReader> {
    player: MpsPlayer<T>,
    control: Receiver<ControlAction>,
    event: Sender<PlayerAction>,
    keep_alive: bool,
}

impl<T: MpsTokenReader> MpsPlayerServer<T> {
    pub fn new(player: MpsPlayer<T>, ctrl: Receiver<ControlAction>, event: Sender<PlayerAction>, keep_alive: bool) -> Self {
        Self {
            player: player,
            control: ctrl,
            event: event,
            keep_alive: keep_alive,
        }
    }

    fn run_loop(&mut self) {
        // this can panic since it's not on the main thread
        // initial queue full
        if let Err(e) = self.player.enqueue(1) {
            self.event.send(PlayerAction::Exception(e)).unwrap();
        }
        let mut is_empty = self.player.queue_len() == 0;
        loop {
            let command = self.control.recv().unwrap();

            let mut is_exiting = false;

            let mut check_empty = false;

            // process command
            match command {
                ControlAction::Next{..} => {
                    //println!("Executing next command (queue_len: {})", self.player.queue_len());
                    if let Err(e) = self.player.new_sink() {
                        self.event.send(PlayerAction::Exception(e)).unwrap();
                    }
                    if !self.player.is_paused() {
                        self.player.enqueue(1).unwrap();
                    }
                },
                ControlAction::Previous{..} => {}, // TODO
                ControlAction::Play{..} => self.player.resume(),
                ControlAction::Pause{..} => self.player.pause(),
                ControlAction::PlayPause{..} => {
                    if self.player.is_paused() {
                        self.player.resume();
                    } else {
                        self.player.pause();
                    }
                },
                ControlAction::Stop{..} => self.player.stop(),
                ControlAction::Exit{..} => {
                    self.player.stop();
                    is_exiting = true;
                },
                ControlAction::Enqueue{amount,..} => {
                    if let Err(e) = self.player.enqueue(amount) {
                        self.event.send(PlayerAction::Exception(e)).unwrap();
                    }
                },
                ControlAction::NoOp{..} => {}, // empty by design
                ControlAction::SetVolume{volume,..} => {
                    self.player.set_volume((volume as f32) / (u32::MAX as f32));
                },
                ControlAction::CheckEmpty{..} => {
                    check_empty = true;
                }
            }

            // keep queue full (while playing music)
            if self.player.queue_len() == 0 && !self.player.is_paused() && !is_exiting {
                if let Err(e) = self.player.enqueue(1) {
                    self.event.send(PlayerAction::Exception(e)).unwrap();
                }
                if self.player.queue_len() == 0  { // no more music to add
                    is_exiting = !self.keep_alive || is_exiting;
                }
            }

            if command.needs_ack() {
                self.event.send(PlayerAction::Acknowledge(command)).unwrap();
            }

            // always check for empty state change
            if self.player.queue_len() == 0 && !is_empty { // just became empty
                is_empty = true;
                self.event.send(PlayerAction::Empty).unwrap();
            } else if self.player.queue_len() != 0 && is_empty { // just got filled
                is_empty = false;
            }

            if is_empty && check_empty {
                self.event.send(PlayerAction::Empty).unwrap();
            }

            if is_exiting { break; }
        }
        println!("Exiting playback server");
        self.event.send(PlayerAction::End).unwrap();
    }

    pub fn spawn<F: FnOnce() -> MpsPlayer<T> + Send + 'static>(
        factory: F,
        ctrl_tx: Sender<ControlAction>,
        ctrl_rx: Receiver<ControlAction>,
        event: Sender<PlayerAction>,
        keep_alive: bool
    ) -> JoinHandle<()> {
        thread::spawn(move || Self::unblocking_timer_loop(ctrl_tx, 50));
        thread::spawn(move || {
            let player = factory();
            let mut server_obj = Self::new(player, ctrl_rx, event, keep_alive);
            server_obj.run_loop();
        })
    }

    pub fn unblocking_timer_loop(ctrl_tx: Sender<ControlAction>, sleep_ms: u64) {
        let dur = std::time::Duration::from_millis(sleep_ms);
        loop {
            if let Err(_) = ctrl_tx.send(ControlAction::NoOp{ack: false}) {
                break;
            }
            thread::sleep(dur);
        }
    }
}

/// Action the controller wants the player to perform
#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ControlAction {
    Next{ack: bool},
    Previous{ack: bool},
    Play{ack: bool},
    Pause{ack: bool},
    PlayPause{ack: bool},
    Stop{ack: bool},
    Exit{ack: bool},
    Enqueue {amount: usize, ack: bool},
    NoOp{ack: bool},
    SetVolume{ack: bool, volume: u32},
    CheckEmpty{ack: bool},
}

impl ControlAction {
    fn needs_ack(&self) -> bool {
        *match self {
            Self::Next{ack} => ack,
            Self::Previous{ack} => ack,
            Self::Play{ack} => ack,
            Self::Pause{ack} => ack,
            Self::PlayPause{ack} => ack,
            Self::Stop{ack} => ack,
            Self::Exit{ack} => ack,
            Self::Enqueue{ack,..} => ack,
            Self::NoOp{ack,..} => ack,
            Self::SetVolume{ack,..} => ack,
            Self::CheckEmpty{ack} => ack,
        }
    }
}

/// Action the player has performed/encountered
#[derive(Clone, Debug)]
pub enum PlayerAction {
    Acknowledge(ControlAction),
    Exception(PlaybackError),
    End,
    Empty,
}

impl PlayerAction {
    pub fn is_acknowledgement(&self) -> bool {
        match self {
            Self::Acknowledge(_) => true,
            _ => false
        }
    }
}
