use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread::JoinHandle;

use mps_interpreter::tokens::MpsTokenReader;

use super::MpsPlayer;
use super::PlaybackError;
use super::player_wrapper::{ControlAction, PlayerAction, MpsPlayerServer};
use super::os_controls::SystemControlWrapper;

pub struct MpsController {
    control: Sender<ControlAction>,
    event: Receiver<PlayerAction>,
    handle: JoinHandle<()>,
    sys_ctrl: SystemControlWrapper
}

impl MpsController {
    pub fn create<F: FnOnce() -> MpsPlayer<T> + Send + 'static, T: MpsTokenReader>(
        player_gen: F
    ) -> Self {
        let (control_tx, control_rx) = channel();
        let (event_tx, event_rx) = channel();
        let mut sys_ctrl = SystemControlWrapper::new(control_tx.clone());
        sys_ctrl.init();
        let handle = MpsPlayerServer::spawn(player_gen, control_tx.clone(), control_rx, event_tx);
        Self {
            control: control_tx,
            event: event_rx,
            handle: handle,
            sys_ctrl: sys_ctrl,
        }
    }

    fn send_confirm(&self, to_send: ControlAction) -> Result<(), PlaybackError> {
        self.control.send(to_send.clone()).map_err(PlaybackError::from_err)?;
        let mut response = self.event.recv().map_err(PlaybackError::from_err)?;
        while !response.is_acknowledgement() {
            Self::handle_event(response)?;
            response = self.event.recv().map_err(PlaybackError::from_err)?;
        }
        if let PlayerAction::Acknowledge(action) = response {
            if action == to_send {
                Ok(())
            } else {
                Err(PlaybackError {
                    msg: "Incorrect acknowledgement received for MpsController control action".into()
                })
            }
        } else {
            Err(PlaybackError {
                msg: "Invalid acknowledgement received for MpsController control action".into()
            })
        }
    }

    fn handle_event(event: PlayerAction) -> Result<(), PlaybackError> {
        match event {
            PlayerAction::Acknowledge(_) => Ok(()),
            PlayerAction::Exception(e) => Err(e),
            PlayerAction::End => Ok(())
        }
    }

    pub fn next(&self) -> Result<(), PlaybackError> {
        self.send_confirm(ControlAction::Next{ack: true})
    }

    pub fn previous(&self) -> Result<(), PlaybackError> {
        self.send_confirm(ControlAction::Previous{ack: true})
    }

    pub fn play(&self) -> Result<(), PlaybackError> {
        self.send_confirm(ControlAction::Play{ack: true})
    }

    pub fn pause(&self) -> Result<(), PlaybackError> {
        self.send_confirm(ControlAction::Pause{ack: true})
    }

    pub fn stop(&self) -> Result<(), PlaybackError> {
        self.send_confirm(ControlAction::Stop{ack: true})
    }

    pub fn enqueue(&self, count: usize) -> Result<(), PlaybackError> {
        self.send_confirm(ControlAction::Enqueue{amount: count, ack: true})
    }

    pub fn ping(&self) -> Result<(), PlaybackError> {
        self.send_confirm(ControlAction::NoOp{ack: true})
    }

    pub fn exit(self) -> Result<(), PlaybackError> {
        self.send_confirm(ControlAction::Exit{ack: true})?;
        self.sys_ctrl.exit();
        match self.handle.join() {
            Ok(x) => Ok(x),
            Err(_) => Err(PlaybackError {
                msg: "MpsPlayerServer did not exit correctly".into()
            })
        }
    }

    pub fn wait_for_done(&self) -> Result<(), PlaybackError> {
        loop {
            let msg = self.event.recv().map_err(PlaybackError::from_err)?;
            if let PlayerAction::End = msg {
                break;
            } else {
                Self::handle_event(msg)?;
            }
        }
        Ok(())
    }
}
