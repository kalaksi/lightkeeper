/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::error::LkError;
use crate::frontend;
use crate::remote_core::protocol::{ServerMessage, write_message};

const STOP_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub struct RemoteSession {
    writer: Arc<Mutex<UnixStream>>,
    stop_sender: Option<mpsc::Sender<()>>,
    update_thread: Option<thread::JoinHandle<()>>,
}

impl RemoteSession {
    pub fn new(stream: UnixStream) -> Self {
        RemoteSession {
            writer: Arc::new(Mutex::new(stream)),
            stop_sender: None,
            update_thread: None,
        }
    }

    pub fn send_message(&self, message: &ServerMessage) -> Result<(), LkError> {
        let mut writer = self.writer.lock()?;
        write_message(&mut *writer, message)?;
        Ok(())
    }

    pub fn send_error<Stringable: ToString>(&self, message: Stringable) -> Result<(), LkError> {
        self.send_message(&ServerMessage::Error {
            request_id: None,
            message: message.to_string(),
        })
    }

    pub fn start_update_stream(&mut self, receiver: mpsc::Receiver<frontend::UIUpdate>) {
        let (stop_sender, stop_receiver) = mpsc::channel();
        let writer = self.writer.clone();
        let thread = thread::spawn(move || {
            loop {
                match stop_receiver.try_recv() {
                    Ok(()) | Err(mpsc::TryRecvError::Disconnected) => return,
                    Err(mpsc::TryRecvError::Empty) => {}
                }

                let update = match receiver.recv_timeout(STOP_POLL_INTERVAL) {
                    Ok(update) => update,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                };

                let result = match update {
                    frontend::UIUpdate::Host(host_update) => {
                        Self::send_message_internal(&writer, &ServerMessage::HostUpdate(host_update))
                    }
                    frontend::UIUpdate::FatalError() => {
                        let result = Self::send_error_internal(&writer, "Core runtime crashed");
                        if result.is_ok() {
                            log::error!("Stopping client session after a fatal core error");
                        }
                        result
                    }
                    frontend::UIUpdate::Stop() => return,
                    frontend::UIUpdate::Chart(_) => continue,
                };

                match result {
                    Ok(()) => {}
                    Err(error) => {
                        log::debug!("Stopping remote session update stream: {}", error);
                        return;
                    }
                }
            }
        });

        self.stop_sender = Some(stop_sender);
        self.update_thread = Some(thread);
    }

    fn stop(&mut self) {
        if let Some(stop_sender) = self.stop_sender.take() {
            let _ = stop_sender.send(());
        }

        if let Some(thread) = self.update_thread.take() {
            if let Err(error) = thread.join() {
                log::error!("Remote session update thread failed: {:?}", error);
            }
        }
    }

    fn send_message_internal(
        writer: &Arc<Mutex<UnixStream>>,
        message: &ServerMessage,
    ) -> Result<(), LkError> {
        let mut writer = writer.lock()?;
        write_message(&mut *writer, message)?;
        Ok(())
    }

    fn send_error_internal<Stringable: ToString>(
        writer: &Arc<Mutex<UnixStream>>,
        message: Stringable,
    ) -> Result<(), LkError> {
        Self::send_message_internal(
            writer,
            &ServerMessage::Error {
                request_id: None,
                message: message.to_string(),
            },
        )
    }
}

impl Drop for RemoteSession {
    fn drop(&mut self) {
        self.stop();
    }
}
