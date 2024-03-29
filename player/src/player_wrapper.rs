use std::sync::mpsc::{Receiver, Sender};
use std::{thread, thread::JoinHandle};

use rodio::Source;

//use muss_interpreter::tokens::TokenReader;
use muss_interpreter::{InterpreterError, Item};

use super::Player;
use super::PlayerError;

/// A wrapper around Player so that playback can occur on a different thread.
/// This allows for message passing between the player and controller.
///
/// You will probably never directly interact with this, instead using Controller to communicate.
pub struct PlayerServer<I: std::iter::Iterator<Item = Result<Item, InterpreterError>>> {
    player: Player<I>,
    control: Receiver<ControlAction>,
    event: Sender<PlayerAction>,
    playback: Sender<PlaybackAction>,
    keep_alive: bool,
}

impl<I: std::iter::Iterator<Item = Result<Item, InterpreterError>>> PlayerServer<I> {
    pub fn new(
        player: Player<I>,
        ctrl: Receiver<ControlAction>,
        event: Sender<PlayerAction>,
        playback: Sender<PlaybackAction>,
        keep_alive: bool,
    ) -> Self {
        Self {
            player: player,
            control: ctrl,
            event: event,
            playback: playback,
            keep_alive: keep_alive,
        }
    }

    fn modify(&self) -> impl Fn(rodio::Decoder<std::io::BufReader<std::fs::File>>, Item) -> Box<dyn Source<Item=i16> + Send> {
        let event = std::sync::Arc::new(std::sync::Mutex::new(self.playback.clone()));
        move |source_in, item| {
            let event2 = event.clone();
            if let Some(duration) = source_in.total_duration() {
                event.lock().map(|event|
                    event.send(
                        PlaybackAction::Time(item.clone(), duration)
                    ).unwrap_or(())
                ).unwrap_or(());
                Box::new(
                    source_in.periodic_access(
                        std::time::Duration::from_secs(1),
                        move |_|
                            {
                                //println!("Debug tick");
                                event2.lock()
                                    .map(|x|
                                        x.send(PlaybackAction::UpdateTick(item.clone())).unwrap_or(())
                                    )
                                    .unwrap_or(());
                            }
                    )
                )
            } else {
                // manually calculate length
                let source_in = source_in.buffered();
                let source_in2 = source_in.clone();
                let event3 = event.clone();
                let item2 = item.clone();
                // Iterator.count() takes a while, so calculate in a different thread
                std::thread::spawn(move || {
                    let sample_rate = source_in2.sample_rate();
                    let channels = source_in2.channels() as u32;
                    let sample_count = source_in2.count() as f64;
                    let duration = std::time::Duration::from_secs_f64(sample_count / ((sample_rate * channels) as f64));
                    event3.lock().map(|event|
                        event.send(
                            PlaybackAction::Time(item2.clone(), duration)
                        ).unwrap_or(())
                    ).unwrap_or(());
                });
                Box::new(
                    source_in.periodic_access(
                        std::time::Duration::from_secs(1),
                        move |_|
                            {
                                event2.lock()
                                    .map(|x|
                                        x.send(PlaybackAction::UpdateTick(item.clone())).unwrap_or(())
                                    )
                                    .unwrap_or(());
                            }
                    )
                )
            }
        }
    }

    fn enqeue_some(&mut self, count: usize) {
        //println!("Enqueuing up to {} items", count);
        match self.player.enqueue_modified(count, &self.modify()) {
            Err(e) => {
                self.event.send(PlayerAction::Exception(e)).unwrap();
            }
            Ok(items) => {
                for item in items {
                    // notify of new items that have been enqueued
                    self.playback.send(PlaybackAction::Enqueued(item)).unwrap();
                }
            }
        }
    }

    fn on_empty(&self) {
        self.event.send(PlayerAction::Empty).unwrap();
        self.playback.send(PlaybackAction::Empty).unwrap();
    }

    fn on_end(&self) {
        self.event.send(PlayerAction::End).unwrap();
        self.playback.send(PlaybackAction::Exit).unwrap();
    }

    fn run_loop(&mut self) {
        // this can panic since it's not on the main thread
        // initial queue fill
        self.enqeue_some(1);
        let mut is_empty = self.player.queue_len() == 0;
        loop {
            let command = self.control.recv().unwrap();

            let mut is_exiting = false;

            let mut check_empty = false;

            // process command
            match command {
                ControlAction::Next { .. } => {
                    //println!("Executing next command (queue_len: {})", self.player.queue_len());
                    if let Err(e) = self.player.new_sink() {
                        self.event.send(PlayerAction::Exception(e)).unwrap();
                    }
                    if !self.player.is_paused() {
                        self.enqeue_some(1);
                    }
                }
                ControlAction::Previous { .. } => {} // TODO
                ControlAction::Play { .. } => self.player.resume(),
                ControlAction::Pause { .. } => self.player.pause(),
                ControlAction::PlayPause { .. } => {
                    if self.player.is_paused() {
                        self.player.resume();
                    } else {
                        self.player.pause();
                    }
                }
                ControlAction::Stop { .. } => self.player.stop(),
                ControlAction::Exit { .. } => {
                    self.player.stop();
                    is_exiting = true;
                }
                ControlAction::Enqueue { amount, .. } => {
                    self.enqeue_some(amount);
                }
                ControlAction::NoOp { .. } => {} // empty by design
                ControlAction::SetVolume { volume, .. } => {
                    self.player.set_volume((volume as f32) / (u32::MAX as f32));
                }
                ControlAction::CheckEmpty { .. } => {
                    check_empty = true;
                }
            }

            // keep queue full (while playing music)
            if self.player.queue_len() == 0 && !self.player.is_paused() && !is_exiting {
                self.enqeue_some(1);
                if self.player.queue_len() == 0 {
                    // no more music to add
                    is_exiting = !self.keep_alive || is_exiting;
                }
            }

            if command.needs_ack() {
                self.event.send(PlayerAction::Acknowledge(command)).unwrap();
            }

            // always check for empty state change
            if self.player.queue_len() == 0 && !is_empty {
                // just became empty
                is_empty = true;
                self.on_empty();
            } else if self.player.queue_len() != 0 && is_empty {
                // just got filled
                is_empty = false;
            }

            if is_empty && check_empty {
                self.on_empty();
            }

            if is_exiting {
                break;
            }
        }
        //println!("Exiting playback server");
        self.on_end();
    }

    pub fn spawn<F: FnOnce() -> Player<I> + Send + 'static>(
        factory: F,
        ctrl_tx: Sender<ControlAction>,
        ctrl_rx: Receiver<ControlAction>,
        event: Sender<PlayerAction>,
        playback: Sender<PlaybackAction>,
        keep_alive: bool,
    ) -> JoinHandle<()> {
        thread::spawn(move || Self::unblocking_timer_loop(ctrl_tx, 100));
        thread::spawn(move || {
            let player = factory();
            let mut server_obj = Self::new(player, ctrl_rx, event, playback, keep_alive);
            server_obj.run_loop();
        })
    }

    pub fn unblocking_timer_loop(ctrl_tx: Sender<ControlAction>, sleep_ms: u64) {
        let dur = std::time::Duration::from_millis(sleep_ms);
        loop {
            if ctrl_tx.send(ControlAction::NoOp { ack: false }).is_err() {
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
    Next { ack: bool },
    Previous { ack: bool },
    Play { ack: bool },
    Pause { ack: bool },
    PlayPause { ack: bool },
    Stop { ack: bool },
    Exit { ack: bool },
    Enqueue { amount: usize, ack: bool },
    NoOp { ack: bool },
    SetVolume { ack: bool, volume: u32 },
    CheckEmpty { ack: bool },
}

impl ControlAction {
    fn needs_ack(&self) -> bool {
        *match self {
            Self::Next { ack } => ack,
            Self::Previous { ack } => ack,
            Self::Play { ack } => ack,
            Self::Pause { ack } => ack,
            Self::PlayPause { ack } => ack,
            Self::Stop { ack } => ack,
            Self::Exit { ack } => ack,
            Self::Enqueue { ack, .. } => ack,
            Self::NoOp { ack, .. } => ack,
            Self::SetVolume { ack, .. } => ack,
            Self::CheckEmpty { ack } => ack,
        }
    }
}

/// Action the player has performed/encountered
#[derive(Clone, Debug)]
pub enum PlayerAction {
    Acknowledge(ControlAction),
    Exception(PlayerError),
    End,
    Empty,
}

#[derive(Clone, Debug)]
pub enum PlaybackAction {
    Empty,
    Enqueued(Item),
    Time(Item, std::time::Duration),
    UpdateTick(Item), // tick sent once every second
    Exit,
}

impl PlayerAction {
    pub fn is_acknowledgement(&self) -> bool {
        match self {
            Self::Acknowledge(_) => true,
            _ => false,
        }
    }
}
