/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::io;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::api::CommandBackend;
use crate::command_handler::CommandButtonData;
use crate::configuration;
use crate::connection_manager::ConnectorRequest;
use crate::error::{ErrorKind, LkError};
use crate::frontend;
use crate::host_manager::StateUpdateMessage;
use crate::remote_core::protocol::{read_message, write_message, ClientMessage, ServerMessage, PROTOCOL_VERSION};
use crate::utils::sha256;

//
// CommandBackend client for lightkeeper-core (unix socket).
//

const REMOTE_READ_TIMEOUT: Duration = Duration::from_millis(100);
const REMOTE_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingRpcKind {
    ExecuteCommand,
    CommandsForHost,
    CommandForHost,
    CustomCommandsForHost,
    AllHostCategories,
    RefreshInvocationIds,
    InitializeHosts,
    ResolveTextEditorPath,
    DownloadEditable,
    UploadEdited,
    WriteCachedFile,
    RemoveCachedFile,
    HasCachedFileChanged,
    UploadFromCache,
}

#[allow(clippy::large_enum_variant)]
enum PendingRpcReply {
    ExecuteCommand(u64),
    CommandsForHost(HashMap<String, CommandButtonData>),
    CommandForHost(Option<CommandButtonData>),
    CustomCommandsForHost(HashMap<String, configuration::CustomCommandConfig>),
    AllHostCategories(Vec<String>),
    RefreshInvocationIds(Vec<u64>),
    InitializeHosts(Vec<String>),
    ResolveTextEditorPath(Option<String>),
    DownloadEditable(u64),
    UploadEdited(u64),
    FileOpDone,
    FileChanged(bool),
    UploadFromCache(u64),
}

struct PendingRpc {
    kind: PendingRpcKind,
    sender: mpsc::Sender<PendingRpcReply>,
}

fn default_reply(kind: PendingRpcKind) -> PendingRpcReply {
    match kind {
        PendingRpcKind::ExecuteCommand => PendingRpcReply::ExecuteCommand(0),
        PendingRpcKind::CommandsForHost => PendingRpcReply::CommandsForHost(HashMap::new()),
        PendingRpcKind::CommandForHost => PendingRpcReply::CommandForHost(None),
        PendingRpcKind::CustomCommandsForHost => PendingRpcReply::CustomCommandsForHost(HashMap::new()),
        PendingRpcKind::AllHostCategories => PendingRpcReply::AllHostCategories(Vec::new()),
        PendingRpcKind::RefreshInvocationIds => PendingRpcReply::RefreshInvocationIds(Vec::new()),
        PendingRpcKind::InitializeHosts => PendingRpcReply::InitializeHosts(Vec::new()),
        PendingRpcKind::ResolveTextEditorPath => PendingRpcReply::ResolveTextEditorPath(None),
        PendingRpcKind::DownloadEditable => PendingRpcReply::DownloadEditable(0),
        PendingRpcKind::UploadEdited => PendingRpcReply::UploadEdited(0),
        PendingRpcKind::WriteCachedFile => PendingRpcReply::FileOpDone,
        PendingRpcKind::RemoveCachedFile => PendingRpcReply::FileOpDone,
        PendingRpcKind::HasCachedFileChanged => PendingRpcReply::FileChanged(false),
        PendingRpcKind::UploadFromCache => PendingRpcReply::UploadFromCache(0),
    }
}

fn completer_send_default(p: PendingRpc) {
    let _ = p.sender.send(default_reply(p.kind));
}

fn reply_matches(kind: &PendingRpcKind, reply: &PendingRpcReply) -> bool {
    matches!(
        (kind, reply),
        (PendingRpcKind::ExecuteCommand, PendingRpcReply::ExecuteCommand(_)) |
            (PendingRpcKind::CommandsForHost, PendingRpcReply::CommandsForHost(_)) |
            (PendingRpcKind::CommandForHost, PendingRpcReply::CommandForHost(_)) |
            (PendingRpcKind::CustomCommandsForHost, PendingRpcReply::CustomCommandsForHost(_)) |
            (PendingRpcKind::AllHostCategories, PendingRpcReply::AllHostCategories(_)) |
            (PendingRpcKind::RefreshInvocationIds, PendingRpcReply::RefreshInvocationIds(_)) |
            (PendingRpcKind::InitializeHosts, PendingRpcReply::InitializeHosts(_)) |
            (PendingRpcKind::ResolveTextEditorPath, PendingRpcReply::ResolveTextEditorPath(_)) |
            (PendingRpcKind::DownloadEditable, PendingRpcReply::DownloadEditable(_)) |
            (PendingRpcKind::UploadEdited, PendingRpcReply::UploadEdited(_)) |
            (PendingRpcKind::WriteCachedFile, PendingRpcReply::FileOpDone) |
            (PendingRpcKind::RemoveCachedFile, PendingRpcReply::FileOpDone) |
            (PendingRpcKind::HasCachedFileChanged, PendingRpcReply::FileChanged(_)) |
            (PendingRpcKind::UploadFromCache, PendingRpcReply::UploadFromCache(_))
    )
}

fn align_reply(kind: PendingRpcKind, reply: PendingRpcReply) -> PendingRpcReply {
    if reply_matches(&kind, &reply) {
        reply
    }
    else {
        ::log::error!("Request failed: unexpected reply");
        default_reply(kind)
    }
}

fn deliver_response(
    pending_rpc: &Arc<Mutex<HashMap<u64, PendingRpc>>>,
    request_id: u64,
    expected: PendingRpcKind,
    on_match: impl FnOnce() -> PendingRpcReply,
) {
    let pending = match pending_rpc.lock() {
        Ok(mut map) => map.remove(&request_id),
        Err(error) => {
            ::log::error!("Request failed: {}", error);
            return;
        }
    };
    match pending {
        None => {
            ::log::error!("Received unexpected response");
        }
        Some(p) if p.kind == expected => {
            if p.sender.send(on_match()).is_err() {
                ::log::error!("Receiver dropped");
            }
        }
        Some(p) => {
            ::log::error!("RPC type mismatch");
            let _ = p.sender.send(default_reply(p.kind));
        }
    }
}

pub struct RemoteCommandBackend {
    socket_path: PathBuf,
    writer: Option<Arc<Mutex<UnixStream>>>,
    pending_rpc: Arc<Mutex<HashMap<u64, PendingRpc>>>,
    next_request_id: Arc<AtomicU64>,
    frontend_update_sender: Option<mpsc::Sender<frontend::UIUpdate>>,
    stop_sender: Option<mpsc::Sender<()>>,
    response_thread: Option<thread::JoinHandle<()>>,
}

impl RemoteCommandBackend {
    pub fn new(socket_path: PathBuf) -> Self {
        RemoteCommandBackend {
            socket_path,
            writer: None,
            pending_rpc: Arc::new(Mutex::new(HashMap::new())),
            next_request_id: Arc::new(AtomicU64::new(1)),
            frontend_update_sender: None,
            stop_sender: None,
            response_thread: None,
        }
    }

    fn next_request_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, Ordering::SeqCst)
    }

    fn connect(&mut self) -> Result<(), String> {
        if self.writer.is_some() {
            return Ok(());
        }

        let stream = UnixStream::connect(&self.socket_path).map_err(|error| error.to_string())?;
        self.connect_stream(stream)
    }

    /// Runs the remote client protocol on an already connected stream (Connect message, reader thread).
    /// [`Self::connect`] uses this after path connect; [`Self::connect_with_frontend_stream`] sets the UI sender
    /// then calls this (e.g. with a stream from `UnixStream::pair`).
    pub fn connect_stream(&mut self, mut stream: UnixStream) -> Result<(), String> {
        if self.writer.is_some() {
            return Ok(());
        }

        let frontend_update_sender = self
            .frontend_update_sender
            .clone()
            .ok_or_else(|| String::from("Missing UI update sender"))?;

        write_message(
            &mut stream,
            &ClientMessage::Connect {
                protocol_version: PROTOCOL_VERSION,
            },
        )
        .map_err(|error| error.to_string())?;

        let mut reader = stream.try_clone().map_err(|error| error.to_string())?;
        reader
            .set_read_timeout(Some(REMOTE_READ_TIMEOUT))
            .map_err(|error| error.to_string())?;

        let writer = Arc::new(Mutex::new(stream));
        let pending_rpc = self.pending_rpc.clone();
        let (stop_sender, stop_receiver) = mpsc::channel();
        let response_thread = thread::spawn(move || loop {
            match stop_receiver.try_recv() {
                Ok(()) | Err(mpsc::TryRecvError::Disconnected) => return,
                Err(mpsc::TryRecvError::Empty) => {}
            }

            let message = match read_message::<ServerMessage, _>(&mut reader) {
                Ok(message) => message,
                Err(error) if error.kind() == io::ErrorKind::TimedOut || error.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(error) => {
                    ::log::error!("Receive failed: {}", error);
                    return;
                }
            };

            match message {
                ServerMessage::Connect { protocol_version } => {
                    if protocol_version != PROTOCOL_VERSION {
                        ::log::error!("Protocol mismatch: expected {}, got {}", PROTOCOL_VERSION, protocol_version,);
                    }
                }
                ServerMessage::ExecuteCommand { request_id, invocation_id } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::ExecuteCommand, || {
                        PendingRpcReply::ExecuteCommand(invocation_id)
                    });
                }
                ServerMessage::CommandsForHost {
                    request_id,
                    host_id: _,
                    commands,
                } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::CommandsForHost, || {
                        PendingRpcReply::CommandsForHost(commands)
                    });
                }
                ServerMessage::CommandForHost {
                    request_id,
                    host_id: _,
                    command_id: _,
                    command,
                } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::CommandForHost, || {
                        PendingRpcReply::CommandForHost(command)
                    });
                }
                ServerMessage::CustomCommandsForHost {
                    request_id,
                    host_id: _,
                    commands,
                } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::CustomCommandsForHost, || {
                        PendingRpcReply::CustomCommandsForHost(commands)
                    });
                }
                ServerMessage::AllHostCategories {
                    request_id,
                    host_id: _,
                    categories,
                } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::AllHostCategories, || {
                        PendingRpcReply::AllHostCategories(categories)
                    });
                }
                ServerMessage::InitialState(display_data) => {
                    for host_display_data in display_data.hosts.into_values() {
                        if frontend_update_sender.send(frontend::UIUpdate::Host(host_display_data)).is_err() {
                            ::log::error!("Failed to deliver initial state update");
                            return;
                        }
                    }
                }
                ServerMessage::HostUpdate(host_display_data) => {
                    if frontend_update_sender.send(frontend::UIUpdate::Host(host_display_data)).is_err() {
                        ::log::error!("Failed to deliver host update");
                        return;
                    }
                }
                ServerMessage::VerificationRequest(request) => {
                    ::log::warn!(
                        "Ignoring standalone verification request for {}: {}",
                        request.source_id,
                        request.message,
                    );
                }
                ServerMessage::RefreshInvocationIds {
                    request_id,
                    invocation_ids,
                } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::RefreshInvocationIds, || {
                        PendingRpcReply::RefreshInvocationIds(invocation_ids)
                    });
                }
                ServerMessage::InitializeHostsResult { request_id, host_ids } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::InitializeHosts, || {
                        PendingRpcReply::InitializeHosts(host_ids)
                    });
                }
                ServerMessage::ResolveTextEditorPath { request_id, path } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::ResolveTextEditorPath, || {
                        PendingRpcReply::ResolveTextEditorPath(path)
                    });
                }
                ServerMessage::DownloadEditableFileResult { request_id, invocation_id } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::DownloadEditable, || {
                        PendingRpcReply::DownloadEditable(invocation_id)
                    });
                }
                ServerMessage::UploadEditedFileResult { request_id, invocation_id } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::UploadEdited, || {
                        PendingRpcReply::UploadEdited(invocation_id)
                    });
                }
                ServerMessage::WriteCachedFileResult { request_id } | ServerMessage::RemoveCachedFileResult { request_id } => {
                    let pending = match pending_rpc.lock() {
                        Ok(mut map) => map.remove(&request_id),
                        Err(error) => {
                            ::log::error!("Request failed: {}", error);
                            continue;
                        }
                    };
                    match pending {
                        None => {
                            ::log::error!("Received unexpected response");
                        }
                        Some(p) if p.kind == PendingRpcKind::WriteCachedFile || p.kind == PendingRpcKind::RemoveCachedFile => {
                            let _ = p.sender.send(PendingRpcReply::FileOpDone);
                        }
                        Some(p) => {
                            ::log::error!("RPC type mismatch");
                            let _ = p.sender.send(default_reply(p.kind));
                        }
                    }
                }
                ServerMessage::HasCachedFileChangedResult { request_id, changed } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::HasCachedFileChanged, || {
                        PendingRpcReply::FileChanged(changed)
                    });
                }
                ServerMessage::UploadFileFromCacheResult { request_id, invocation_id } => {
                    deliver_response(&pending_rpc, request_id, PendingRpcKind::UploadFromCache, || {
                        PendingRpcReply::UploadFromCache(invocation_id)
                    });
                }
                ServerMessage::Error { request_id, message } => {
                    ::log::error!("Core server error: {}", message);
                    if let Some(request_id) = request_id {
                        match pending_rpc.lock() {
                            Ok(mut map) => {
                                if let Some(pending) = map.remove(&request_id) {
                                    completer_send_default(pending);
                                }
                            }
                            Err(err) => {
                                ::log::error!("Request failed: {}", err);
                            }
                        }
                    }
                }
            }
        });

        self.writer = Some(writer);
        self.stop_sender = Some(stop_sender);
        self.response_thread = Some(response_thread);
        Ok(())
    }

    fn send_nowait(&self, message: &ClientMessage) -> Result<(), String> {
        let writer = self.writer.as_ref().ok_or_else(|| String::from("Not connected to remote core"))?;
        let mut writer = writer.lock().map_err(|error| error.to_string())?;
        write_message(&mut *writer, message).map_err(|error| error.to_string())
    }

    fn send_message_result(
        &self,
        kind: PendingRpcKind,
        build_message: impl FnOnce(u64) -> ClientMessage,
    ) -> Result<PendingRpcReply, LkError> {

        let request_id = self.next_request_id();
        let (sender, receiver) = mpsc::channel();
        let mut map = self.pending_rpc.lock().map_err(LkError::from)?;
        map.insert(request_id, PendingRpc { kind, sender });
        drop(map);

        let message = build_message(request_id);
        if let Err(error) = self.send_nowait(&message) {
            if let Ok(mut map) = self.pending_rpc.lock() {
                if let Some(pending) = map.remove(&request_id) {
                    completer_send_default(pending);
                }
            }
            return Err(LkError::new(ErrorKind::ConnectionFailed, error));
        }

        match receiver.recv_timeout(REMOTE_COMMAND_TIMEOUT) {
            Ok(reply) => {
                if reply_matches(&kind, &reply) {
                    Ok(align_reply(kind, reply))
                }
                else {
                    Err(LkError::other("Unexpected response from remote core"))
                }
            },
            Err(recv_error) => {
                if let Ok(mut map) = self.pending_rpc.lock() {
                    let _ = map.remove(&request_id);
                }
                Err(LkError::new(ErrorKind::ConnectionFailed, recv_error.to_string()))
            },
        }
    }

    fn send_message(&self, kind: PendingRpcKind, build_message: impl FnOnce(u64) -> ClientMessage) -> PendingRpcReply {
        match self.send_message_result(kind, build_message) {
            Ok(reply) => reply,
            Err(err) => {
                ::log::error!("Request failed: {}", err);
                default_reply(kind)
            },
        }
    }

    fn stop_connection(&mut self) {
        if let Some(writer) = &self.writer {
            let disconnect = ClientMessage::Disconnect;
            if let Ok(mut writer) = writer.lock() {
                let _ = write_message(&mut *writer, &disconnect);
            }
        }

        if let Some(stop_sender) = self.stop_sender.take() {
            let _ = stop_sender.send(());
        }

        if let Some(response_thread) = self.response_thread.take() {
            if let Err(error) = response_thread.join() {
                ::log::error!("Response thread failed: {:?}", error);
            }
        }

        self.writer = None;
    }

    /// Sets the UI update sender and runs [`Self::connect_stream`] on `stream` (e.g. from `UnixStream::pair`),
    /// instead of [`Self::connect`] to the socket path.
    pub fn connect_with_frontend_stream(
        &mut self,
        frontend_update_sender: mpsc::Sender<frontend::UIUpdate>,
        stream: UnixStream,
    ) -> Result<(), String> {
        self.frontend_update_sender = Some(frontend_update_sender);
        self.connect_stream(stream)
    }
}

impl CommandBackend for RemoteCommandBackend {
    fn configure(
        &mut self,
        _hosts_config: &configuration::Hosts,
        _preferences: &configuration::Preferences,
        _request_sender: mpsc::Sender<ConnectorRequest>,
        _update_sender: mpsc::Sender<StateUpdateMessage>,
        frontend_update_sender: mpsc::Sender<frontend::UIUpdate>,
    ) {
        self.frontend_update_sender = Some(frontend_update_sender);
        if let Err(error) = self.connect() {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn start_processing_responses(&mut self) {}

    fn stop(&mut self) {
        self.stop_connection();
    }

    fn refresh_host_monitors(&mut self, host_id: &str) {
        if let Err(error) = self.send_nowait(&ClientMessage::RefreshHostMonitors {
            host_id: host_id.to_string(),
        }) {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn commands_for_host(&self, host_id: &str) -> HashMap<String, CommandButtonData> {
        match self.send_message(PendingRpcKind::CommandsForHost, |request_id| ClientMessage::CommandsForHost {
            request_id,
            host_id: host_id.to_string(),
        }) {
            PendingRpcReply::CommandsForHost(commands) => commands,
            _ => HashMap::new(),
        }
    }

    fn command_for_host(&self, host_id: &str, command_id: &str) -> Option<CommandButtonData> {
        match self.send_message(PendingRpcKind::CommandForHost, |request_id| ClientMessage::CommandForHost {
            request_id,
            host_id: host_id.to_string(),
            command_id: command_id.to_string(),
        }) {
            PendingRpcReply::CommandForHost(command) => command,
            _ => None,
        }
    }

    fn custom_commands_for_host(&self, host_id: &str) -> HashMap<String, configuration::CustomCommandConfig> {
        match self.send_message(PendingRpcKind::CustomCommandsForHost, |request_id| {
            ClientMessage::CustomCommandsForHost {
                request_id,
                host_id: host_id.to_string(),
            }
        }) {
            PendingRpcReply::CustomCommandsForHost(commands) => commands,
            _ => HashMap::new(),
        }
    }

    fn all_host_categories(&self, host_id: &str) -> Vec<String> {
        match self.send_message(PendingRpcKind::AllHostCategories, |request_id| ClientMessage::AllHostCategories {
            request_id,
            host_id: host_id.to_string(),
        }) {
            PendingRpcReply::AllHostCategories(categories) => categories,
            _ => Vec::new(),
        }
    }

    fn execute_command(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> u64 {
        match self.send_message(PendingRpcKind::ExecuteCommand, |request_id| ClientMessage::ExecuteCommand {
            request_id,
            host_id: host_id.to_string(),
            command_id: command_id.to_string(),
            parameters: parameters.to_vec(),
        }) {
            PendingRpcReply::ExecuteCommand(id) => id,
            _ => 0,
        }
    }

    fn interrupt_invocation(&self, invocation_id: u64) {
        if self.writer.is_none() {
            return;
        }

        if let Err(error) = self.send_nowait(&ClientMessage::InterruptInvocation { invocation_id }) {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn verify_host_key(&self, host_id: &str, connector_id: &str, key_id: &str) {
        if let Err(error) = self.send_nowait(&ClientMessage::VerifyHostKey {
            host_id: host_id.to_string(),
            connector_id: connector_id.to_string(),
            key_id: key_id.to_string(),
        }) {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn initialize_host(&mut self, host_id: &str) {
        if let Err(error) = self.send_nowait(&ClientMessage::RefreshPlatformInfo {
            host_id: host_id.to_string(),
        }) {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn initialize_hosts(&mut self) -> Vec<String> {
        match self.send_message(PendingRpcKind::InitializeHosts, |request_id| {
            ClientMessage::RefreshPlatformInfoAll { request_id }
        }) {
            PendingRpcReply::InitializeHosts(host_ids) => host_ids,
            _ => Vec::new(),
        }
    }

    fn refresh_monitors_for_command(&mut self, host_id: &str, command_id: &str) -> Vec<u64> {
        match self.send_message(PendingRpcKind::RefreshInvocationIds, |request_id| {
            ClientMessage::RefreshMonitorsForCommand {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
            }
        }) {
            PendingRpcReply::RefreshInvocationIds(invocation_ids) => invocation_ids,
            _ => Vec::new(),
        }
    }

    fn refresh_monitors_of_category(&mut self, host_id: &str, category: &str) -> Vec<u64> {
        match self.send_message(PendingRpcKind::RefreshInvocationIds, |request_id| {
            ClientMessage::RefreshMonitorsOfCategory {
                request_id,
                host_id: host_id.to_string(),
                category: category.to_string(),
            }
        }) {
            PendingRpcReply::RefreshInvocationIds(invocation_ids) => invocation_ids,
            _ => Vec::new(),
        }
    }

    fn refresh_certificate_monitors(&mut self) -> Vec<u64> {
        match self.send_message(PendingRpcKind::RefreshInvocationIds, |request_id| {
            ClientMessage::RefreshCertificateMonitors { request_id }
        }) {
            PendingRpcReply::RefreshInvocationIds(invocation_ids) => invocation_ids,
            _ => Vec::new(),
        }
    }

    fn resolve_text_editor_path(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Option<String> {
        match self.send_message(PendingRpcKind::ResolveTextEditorPath, |request_id| {
            ClientMessage::ResolveTextEditorPath {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
                parameters: parameters.to_vec(),
            }
        }) {
            PendingRpcReply::ResolveTextEditorPath(path) => path,
            _ => None,
        }
    }

    fn download_editable_file(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> (u64, String) {
        let remote_path = remote_file_path.to_string();
        match self.send_message(PendingRpcKind::DownloadEditable, |request_id| ClientMessage::DownloadEditableFile {
            request_id,
            host_id: host_id.to_string(),
            command_id: command_id.to_string(),
            remote_file_path: remote_path.clone(),
        }) {
            PendingRpcReply::DownloadEditable(invocation_id) => (invocation_id, remote_path),
            _ => (0, String::new()),
        }
    }

    fn upload_file(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> u64 {
        match self.send_message(PendingRpcKind::UploadFromCache, |request_id| ClientMessage::UploadFileFromCache {
            request_id,
            host_id: host_id.to_string(),
            command_id: command_id.to_string(),
            remote_file_path: remote_file_path.to_string(),
        }) {
            PendingRpcReply::UploadFromCache(id) => id,
            _ => 0,
        }
    }

    fn upload_file_from_cache(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> u64 {
        self.upload_file(host_id, command_id, remote_file_path)
    }

    fn upload_file_from_editor(&mut self, host_id: &str, command_id: &str, remote_file_path: &str, contents: Vec<u8>) -> u64 {
        match self.send_message(PendingRpcKind::UploadEdited, move |request_id| ClientMessage::UploadEditedFile {
            request_id,
            host_id: host_id.to_string(),
            command_id: command_id.to_string(),
            remote_file_path: remote_file_path.to_string(),
            contents,
        }) {
            PendingRpcReply::UploadEdited(invocation_id) => invocation_id,
            _ => 0,
        }
    }

    fn write_cached_file(&mut self, host_id: &str, remote_file_path: &str, new_contents: Vec<u8>) -> Result<(), LkError> {
        let host_id = host_id.to_string();
        let remote_file_path = remote_file_path.to_string();
        self.send_message_result(PendingRpcKind::WriteCachedFile, move |request_id| ClientMessage::WriteCachedFile {
            request_id,
            host_id,
            remote_file_path,
            contents: new_contents,
        })?;
        Ok(())
    }

    fn remove_cached_file(&mut self, host_id: &str, remote_file_path: &str) -> Result<(), LkError> {
        self.send_message_result(PendingRpcKind::RemoveCachedFile, |request_id| ClientMessage::RemoveCachedFile {
            request_id,
            host_id: host_id.to_string(),
            remote_file_path: remote_file_path.to_string(),
        })?;
        Ok(())
    }

    fn has_cached_file_changed(&self, host_id: &str, remote_file_path: &str, new_contents: &[u8]) -> Result<bool, LkError> {
        let hex = sha256::hash(new_contents);
        match self.send_message_result(PendingRpcKind::HasCachedFileChanged, |request_id| {
            ClientMessage::HasCachedFileChanged {
                request_id,
                host_id: host_id.to_string(),
                remote_file_path: remote_file_path.to_string(),
                content_hash: hex,
            }
        })? {
            PendingRpcReply::FileChanged(changed) => Ok(changed),
            _ => Err(LkError::unexpected()),
        }
    }
}
