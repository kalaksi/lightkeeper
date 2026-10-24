/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::VecDeque;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use crate::error::LkError;
use crate::frontend;
use crate::remote_core::protocol::{write_message, RemoteErrorCode, ServerMessage};

const STOP_POLL_INTERVAL: Duration = Duration::from_millis(100);
/// A client that doesn't read for this long is disconnected.
const WRITE_TIMEOUT: Duration = Duration::from_secs(30);
/// Host updates can't be coalesced since they carry one-off results, so a client that falls this far behind
/// is disconnected instead. It gets full state again on reconnect.
const MAX_QUEUED_UPDATES: usize = 2000;

#[derive(Default)]
struct Outbox {
    /// Replies are written before updates so update traffic doesn't delay RPC responses.
    replies: VecDeque<ServerMessage>,
    updates: VecDeque<ServerMessage>,
    closed: bool,
}

type SharedOutbox = Arc<(Mutex<Outbox>, Condvar)>;

pub struct RemoteSession {
    outbox: SharedOutbox,
    writer_thread: Option<thread::JoinHandle<()>>,
    stop_sender: Option<mpsc::Sender<()>>,
    update_thread: Option<thread::JoinHandle<()>>,
}

impl RemoteSession {
    pub fn new(stream: UnixStream) -> Self {
        if let Err(error) = stream.set_write_timeout(Some(WRITE_TIMEOUT)) {
            log::warn!("Failed to set client write timeout: {}", error);
        }
        let outbox: SharedOutbox = Arc::new((Mutex::new(Outbox::default()), Condvar::new()));
        let writer_outbox = outbox.clone();

        RemoteSession {
            outbox,
            writer_thread: Some(thread::spawn(move || Self::write_loop(stream, writer_outbox))),
            stop_sender: None,
            update_thread: None,
        }
    }

    pub fn send_message(&self, message: &ServerMessage) -> Result<(), LkError> {
        let (lock, condvar) = &*self.outbox;
        let mut outbox = lock.lock()?;
        if outbox.closed {
            return Err(LkError::other("Client session is closed"));
        }
        outbox.replies.push_back(message.clone());
        condvar.notify_one();
        Ok(())
    }

    pub fn send_error(&self, code: RemoteErrorCode, message: impl ToString) -> Result<(), LkError> {
        self.send_message(&ServerMessage::error(None, code, message))
    }

    pub fn send_request_error(&self, request_id: u64, code: RemoteErrorCode, message: impl ToString) -> Result<(), LkError> {
        self.send_message(&ServerMessage::error(Some(request_id), code, message))
    }

    pub fn start_update_stream(&mut self, receiver: mpsc::Receiver<frontend::UIUpdate>) {
        let (stop_sender, stop_receiver) = mpsc::channel();
        let outbox = self.outbox.clone();
        let thread = thread::spawn(move || loop {
            match stop_receiver.try_recv() {
                Ok(()) | Err(mpsc::TryRecvError::Disconnected) => return,
                Err(mpsc::TryRecvError::Empty) => {}
            }

            let update = match receiver.recv_timeout(STOP_POLL_INTERVAL) {
                Ok(update) => update,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            };

            let message = match update {
                frontend::UIUpdate::Host(host_update) => ServerMessage::HostUpdate(host_update),
                frontend::UIUpdate::Chart(response) => ServerMessage::Chart(response),
                frontend::UIUpdate::FatalError() => {
                    log::error!("Stopping client session after a fatal core error");
                    ServerMessage::error(None, RemoteErrorCode::Internal, "Core runtime crashed")
                }
                frontend::UIUpdate::Stop() => return,
                frontend::UIUpdate::CoreConnectionChanged()
                | frontend::UIUpdate::CommandResult(..)
                | frontend::UIUpdate::ReplaceHosts(_) => continue,
            };

            if !Self::queue_update(&outbox, message) {
                log::debug!("Stopping remote session update stream");
                return;
            }
        });

        self.stop_sender = Some(stop_sender);
        self.update_thread = Some(thread);
    }

    pub(crate) fn halt_update_stream(&mut self) {
        self.stop_updates();
    }

    /// Returns false if the session is closed.
    fn queue_update(outbox: &SharedOutbox, message: ServerMessage) -> bool {
        let (lock, condvar) = &**outbox;
        let mut outbox = lock.lock().unwrap();
        if outbox.closed {
            return false;
        }

        if outbox.updates.len() >= MAX_QUEUED_UPDATES {
            log::warn!("Client is not keeping up with updates, disconnecting");
            outbox.updates.clear();
            outbox.replies.push_back(ServerMessage::error(
                None,
                RemoteErrorCode::Internal,
                "Disconnected since the client couldn't keep up with updates",
            ));
            outbox.closed = true;
            condvar.notify_one();
            return false;
        }

        outbox.updates.push_back(message);
        condvar.notify_one();
        true
    }

    /// Writes queued messages until the session is closed and the queue is drained, or writing fails.
    fn write_loop(mut stream: UnixStream, outbox: SharedOutbox) {
        let (lock, condvar) = &*outbox;
        loop {
            let message = {
                let mut outbox = lock.lock().unwrap();
                loop {
                    if let Some(message) = outbox.replies.pop_front().or_else(|| outbox.updates.pop_front()) {
                        break Some(message);
                    }
                    if outbox.closed {
                        break None;
                    }
                    outbox = condvar.wait(outbox).unwrap();
                }
            };

            let Some(message) = message
            else {
                break;
            };

            if let Err(error) = write_message(&mut stream, &message) {
                log::debug!("Stopping remote session writer: {}", error);
                break;
            }
        }

        let mut outbox = lock.lock().unwrap();
        outbox.closed = true;
        outbox.replies.clear();
        outbox.updates.clear();
        drop(outbox);
        // Also ends the session's read loop.
        let _ = stream.shutdown(Shutdown::Both);
    }

    fn stop_updates(&mut self) {
        if let Some(stop_sender) = self.stop_sender.take() {
            let _ = stop_sender.send(());
        }

        if let Some(thread) = self.update_thread.take() {
            if let Err(error) = thread.join() {
                log::error!("Remote session update thread failed: {:?}", error);
            }
        }
    }
}

impl Drop for RemoteSession {
    fn drop(&mut self) {
        self.stop_updates();

        let (lock, condvar) = &*self.outbox;
        if let Ok(mut outbox) = lock.lock() {
            outbox.closed = true;
            outbox.updates.clear();
            condvar.notify_one();
        }
        if let Some(thread) = self.writer_thread.take() {
            if let Err(error) = thread.join() {
                log::error!("Remote session writer thread failed: {:?}", error);
            }
        }
    }
}
