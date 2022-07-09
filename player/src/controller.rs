use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

use muss_interpreter::{Item, InterpreterError};

use super::os_controls::SystemControlWrapper;
use super::player_wrapper::{ControlAction, PlayerServer, PlayerAction};
use super::Player;
use super::PlaybackError;
use super::PlayerError;

/// A controller for a Player running on another thread.
/// This receives and sends events like media buttons and script errors for the Player.
pub struct Controller {
    control: Sender<ControlAction>,
    event: Receiver<PlayerAction>,
    handle: JoinHandle<()>,
    sys_ctrl: SystemControlWrapper,
}

impl Controller {
    pub fn create<
        F: FnOnce() -> Player<I> + Send + 'static,
        I: std::iter::Iterator<Item=Result<Item, InterpreterError>>,
    >(
        player_gen: F,
    ) -> Self {
        let (control_tx, control_rx) = channel();
        let (event_tx, event_rx) = channel();
        let (playback_tx, playback_rx) = channel();
        let mut sys_ctrl = SystemControlWrapper::new(control_tx.clone());
        sys_ctrl.init(playback_rx);
        let handle = PlayerServer::spawn(
            player_gen,
            control_tx.clone(),
            control_rx,
            event_tx,
            playback_tx,
            false,
        );
        Self {
            control: control_tx,
            event: event_rx,
            handle: handle,
            sys_ctrl: sys_ctrl,
        }
    }

    pub fn create_repl<
        F: FnOnce() -> Player<I> + Send + 'static,
        I: std::iter::Iterator<Item=Result<Item, InterpreterError>>,
    >(
        player_gen: F,
    ) -> Self {
        let (control_tx, control_rx) = channel();
        let (event_tx, event_rx) = channel();
        let (playback_tx, playback_rx) = channel();
        let mut sys_ctrl = SystemControlWrapper::new(control_tx.clone());
        sys_ctrl.init(playback_rx);
        let handle = PlayerServer::spawn(
            player_gen,
            control_tx.clone(),
            control_rx,
            event_tx,
            playback_tx,
            true,
        );
        Self {
            control: control_tx,
            event: event_rx,
            handle: handle,
            sys_ctrl: sys_ctrl,
        }
    }

    fn send_confirm(&self, to_send: ControlAction) -> Result<(), PlayerError> {
        self.control
            .send(to_send.clone())
            .map_err(PlayerError::from_err_playback)?;
        let mut response = self.event.recv().map_err(PlayerError::from_err_playback)?;
        while !response.is_acknowledgement() {
            self.handle_event(response)?;
            response = self.event.recv().map_err(PlayerError::from_err_playback)?;
        }
        if let PlayerAction::Acknowledge(action) = response {
            if action == to_send {
                Ok(())
            } else {
                Err(PlaybackError {
                    msg: "Incorrect acknowledgement received for Controller control action"
                        .into(),
                }.into())
            }
        } else {
            Err(PlaybackError {
                msg: "Invalid acknowledgement received for Controller control action".into(),
            }.into())
        }
    }

    fn handle_event(&self, event: PlayerAction) -> Result<(), PlayerError> {
        match event {
            PlayerAction::Acknowledge(_) => Ok(()),
            PlayerAction::Exception(e) => Err(e),
            PlayerAction::End => Ok(()),
            PlayerAction::Empty => Ok(()),
            //PlayerAction::Enqueued(item) => Ok(()),
        }
    }

    pub fn next(&self) -> Result<(), PlayerError> {
        self.send_confirm(ControlAction::Next { ack: true })
    }

    pub fn previous(&self) -> Result<(), PlayerError> {
        self.send_confirm(ControlAction::Previous { ack: true })
    }

    pub fn play(&self) -> Result<(), PlayerError> {
        self.send_confirm(ControlAction::Play { ack: true })
    }

    pub fn pause(&self) -> Result<(), PlayerError> {
        self.send_confirm(ControlAction::Pause { ack: true })
    }

    pub fn stop(&self) -> Result<(), PlayerError> {
        self.send_confirm(ControlAction::Stop { ack: true })
    }

    pub fn enqueue(&self, count: usize) -> Result<(), PlayerError> {
        self.send_confirm(ControlAction::Enqueue {
            amount: count,
            ack: true,
        })
    }

    pub fn ping(&self) -> Result<(), PlayerError> {
        self.send_confirm(ControlAction::NoOp { ack: true })
    }

    pub fn exit(self) -> Result<(), PlayerError> {
        self.send_confirm(ControlAction::Exit { ack: true })?;
        self.sys_ctrl.exit();
        match self.handle.join() {
            Ok(x) => Ok(x),
            Err(_) => Err(PlaybackError {
                msg: "PlayerServer did not exit correctly".into(),
            }.into()),
        }
    }

    pub fn wait_for_done(&self) -> Result<(), PlayerError> {
        loop {
            let msg = self.event.recv().map_err(PlayerError::from_err_playback)?;
            if let PlayerAction::End = msg {
                break;
            } else {
                self.handle_event(msg)?;
            }
        }
        Ok(())
    }

    pub fn wait_for_empty(&self) -> Result<(), PlayerError> {
        for msg in self.event.try_iter() {
            self.handle_event(msg)?;
        }
        self.control
            .send(ControlAction::CheckEmpty { ack: true })
            .map_err(PlayerError::from_err_playback)?;
        loop {
            let msg = self.event.recv().map_err(PlayerError::from_err_playback)?;
            if let PlayerAction::Empty = msg {
                break;
            } else {
                self.handle_event(msg)?;
            }
        }
        Ok(())
    }

    /// Check for any errors in the event queue.
    /// This is non-blocking, so it only handles events sent since the last time events were handled.
    pub fn check(&self) -> Vec<PlayerError> {
        let mut result = Vec::new();
        for msg in self.event.try_iter() {
            if let Err(e) = self.handle_event(msg) {
                result.push(e);
            }
        }
        result
    }

    /// Like check(), but it also waits for an acknowledgement to ensure it gets the latest events.
    pub fn check_ack(&self) -> Vec<PlayerError> {
        let mut result = self.check(); // clear existing messages first
        let to_send = ControlAction::NoOp { ack: true };
        if let Err(e) = self
            .control
            .send(to_send.clone())
            .map_err(PlayerError::from_err_playback)
        {
            result.push(e);
        }
        for msg in self.event.iter() {
            if let PlayerAction::Acknowledge(action) = msg {
                if action == to_send {
                    break;
                } else {
                    result.push(PlaybackError {
                        msg: "Incorrect acknowledgement received for Controller control action"
                            .into(),
                    }.into());
                }
            } else if let Err(e) = self.handle_event(msg) {
                result.push(e);
            }
        }
        result
    }
}
