/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use ssh2;

use super::api::{CommandBackend, ConfigBackend};
use super::core_connection::{CoreConnectionState, CoreConnectionStatus};
use super::remote_config::RemoteConfigBackend;
use super::ssh_transport::Ssh2DirectStreamLocalTransport;
use crate::command_handler::CommandButtonData;
use crate::configuration;
use crate::connection_manager::ConnectorRequest;
use crate::error::{ErrorKind, LkError};
use crate::frontend;
use crate::host_manager::StateUpdateMessage;
use crate::remote_core::protocol::{
    read_message, write_message, ClientMessage, RemoteErrorCode, ServerMessage, PROTOCOL_VERSION,
};
use crate::utils::sha256;

enum CoreClientStream {
    Unix(UnixStream),
    Ssh(ssh2::Channel),
}

impl CoreClientStream {
    fn try_clone(&self) -> Result<Self, String> {
        match self {
            CoreClientStream::Unix(stream) => stream
                .try_clone()
                .map(CoreClientStream::Unix)
                .map_err(|error| error.to_string()),
            CoreClientStream::Ssh(channel) => Ok(CoreClientStream::Ssh(channel.clone())),
        }
    }

    fn set_unix_read_timeout(&self, timeout: Option<Duration>) -> Result<(), String> {
        if let CoreClientStream::Unix(stream) = self {
            stream
                .set_read_timeout(timeout)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

impl Read for CoreClientStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            CoreClientStream::Unix(stream) => stream.read(buf),
            CoreClientStream::Ssh(channel) => channel.read(buf),
        }
    }
}

impl Write for CoreClientStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            CoreClientStream::Unix(stream) => stream.write(buf),
            CoreClientStream::Ssh(channel) => channel.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            CoreClientStream::Unix(stream) => stream.flush(),
            CoreClientStream::Ssh(channel) => channel.flush(),
        }
    }
}

//
// CommandBackend client for lightkeeper-core (unix socket).
//

const REMOTE_READ_TIMEOUT: Duration = Duration::from_millis(100);
const REMOTE_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const REMOTE_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

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
    WriteCachedFile,
    RemoveCachedFile,
    HasCachedFileChanged,
    UploadFromCache,
    Config,
    UpdateConfig,
    GetSecret,
    StoreSecret,
    RemoveSecret,
    Ack,
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
    FileOpDone,
    FileChanged(bool),
    UploadFromCache(u64),
    Config {
        main_yml: String,
        hosts_yml: String,
        groups_yml: String,
    },
    UpdateConfigOk,
    GetSecret(Option<String>),
    StoreSecret(String),
    RemoveSecretOk,
    Ack,
    Error {
        code: RemoteErrorCode,
        message: String,
    },
}

struct PendingRpc {
    kind: PendingRpcKind,
    sender: mpsc::Sender<PendingRpcReply>,
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
            (PendingRpcKind::WriteCachedFile, PendingRpcReply::FileOpDone) |
            (PendingRpcKind::RemoveCachedFile, PendingRpcReply::FileOpDone) |
            (PendingRpcKind::HasCachedFileChanged, PendingRpcReply::FileChanged(_)) |
            (PendingRpcKind::UploadFromCache, PendingRpcReply::UploadFromCache(_)) |
            (PendingRpcKind::Config, PendingRpcReply::Config { .. }) |
            (PendingRpcKind::UpdateConfig, PendingRpcReply::UpdateConfigOk) |
            (PendingRpcKind::GetSecret, PendingRpcReply::GetSecret(_)) |
            (PendingRpcKind::StoreSecret, PendingRpcReply::StoreSecret(_)) |
            (PendingRpcKind::RemoveSecret, PendingRpcReply::RemoveSecretOk) |
            (PendingRpcKind::Ack, PendingRpcReply::Ack)
    )
}

fn internal_error_reply(message: impl ToString) -> PendingRpcReply {
    PendingRpcReply::Error {
        code: RemoteErrorCode::Internal,
        message: message.to_string(),
    }
}

fn fail_all_pending_rpcs(pending_rpc: &Arc<Mutex<HashMap<u64, PendingRpc>>>, code: RemoteErrorCode, message: &str) {
    let pending = match pending_rpc.lock() {
        Ok(mut map) => map.drain().map(|(_, pending)| pending).collect::<Vec<_>>(),
        Err(error) => {
            ::log::error!("Failed to lock pending RPCs: {}", error);
            return;
        }
    };
    for pending in pending {
        let _ = pending.sender.send(PendingRpcReply::Error {
            code,
            message: message.to_string(),
        });
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
            ::log::error!("Received response for unknown request id {}", request_id);
        }
        Some(p) if p.kind == expected => {
            if p.sender.send(on_match()).is_err() {
                ::log::error!("Receiver dropped");
            }
        }
        Some(p) => {
            ::log::error!("RPC type mismatch for request id {}", request_id);
            let _ = p.sender.send(internal_error_reply("RPC response type mismatch"));
        }
    }
}

fn perform_handshake(writer: &mut CoreClientStream, reader: &mut CoreClientStream) -> Result<(), String> {
    write_message(writer, &ClientMessage::Connect { protocol_version: PROTOCOL_VERSION })
        .map_err(|error| error.to_string())?;

    loop {
        match read_message::<ServerMessage, _>(reader) {
            Ok(ServerMessage::Connect { protocol_version }) => {
                if protocol_version != PROTOCOL_VERSION {
                    return Err(format!(
                        "Protocol mismatch: expected {}, got {}",
                        PROTOCOL_VERSION, protocol_version,
                    ));
                }
                return Ok(());
            }
            Ok(ServerMessage::Error { code, message, .. }) => {
                return Err(format!("{}: {}", code, message));
            }
            Ok(_) => {
                return Err(String::from("Unexpected message during remote core handshake"));
            }
            Err(error) if error.kind() == io::ErrorKind::TimedOut || error.kind() == io::ErrorKind::WouldBlock => {
                return Err(String::from("Remote core handshake timed out"));
            }
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                return Err(String::from("Connection closed during remote core handshake"));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

fn open_transport_stream(
    profile: &configuration::CoreConnectionProfile,
    socket_path: &PathBuf,
    stopping: &AtomicBool,
) -> Result<(Option<Ssh2DirectStreamLocalTransport>, CoreClientStream), String> {
    if profile.is_configured() {
        let transport = Ssh2DirectStreamLocalTransport::start(profile, stopping)?;
        let stream = CoreClientStream::Ssh(transport.channel());
        Ok((Some(transport), stream))
    }
    else {
        let stream = UnixStream::connect(socket_path).map_err(|error| error.to_string())?;
        Ok((None, CoreClientStream::Unix(stream)))
    }
}

struct RemoteConnection {
    frontend_update_sender: Option<mpsc::Sender<frontend::UIUpdate>>,
    writer: Option<Arc<Mutex<CoreClientStream>>>,
    stop_sender: Option<mpsc::Sender<()>>,
    response_thread: Option<thread::JoinHandle<()>>,
    transport: Option<Ssh2DirectStreamLocalTransport>,
}

pub struct RemoteCoreClient {
    socket_path: PathBuf,
    connection: Arc<Mutex<RemoteConnection>>,
    pending_rpc: Arc<Mutex<HashMap<u64, PendingRpc>>>,
    next_request_id: Arc<AtomicU64>,
    stopping: Arc<AtomicBool>,
    profile: Mutex<configuration::CoreConnectionProfile>,
    status: Arc<Mutex<CoreConnectionStatus>>,
}

impl RemoteCoreClient {
    pub fn new(socket_path: PathBuf) -> Self {
        RemoteCoreClient {
            socket_path,
            connection: Arc::new(Mutex::new(RemoteConnection {
                frontend_update_sender: None,
                writer: None,
                stop_sender: None,
                response_thread: None,
                transport: None,
            })),
            pending_rpc: Arc::new(Mutex::new(HashMap::new())),
            next_request_id: Arc::new(AtomicU64::new(1)),
            stopping: Arc::new(AtomicBool::new(false)),
            profile: Mutex::new(configuration::CoreConnectionProfile::default()),
            status: Arc::new(Mutex::new(CoreConnectionStatus::default())),
        }
    }

    pub fn set_frontend_update_sender(&self, sender: mpsc::Sender<frontend::UIUpdate>) {
        self.connection.lock().unwrap().frontend_update_sender = Some(sender);
    }

    pub fn profile(&self) -> configuration::CoreConnectionProfile {
        self.profile.lock().unwrap().clone()
    }

    pub fn set_profile(&self, profile: configuration::CoreConnectionProfile) {
        *self.profile.lock().unwrap() = profile;
    }

    pub fn status(&self) -> CoreConnectionStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn connection_state(&self) -> CoreConnectionState {
        self.status.lock().unwrap().state
    }

    pub fn last_error(&self) -> Option<String> {
        self.status.lock().unwrap().last_error.clone()
    }

    fn set_connection_state(&self, state: CoreConnectionState) {
        self.status.lock().unwrap().set_state(state);
    }

    fn set_connection_failed(&self, message: impl Into<String>) {
        self.status.lock().unwrap().set_failed(message);
    }

    fn next_request_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn connect(&self) -> Result<(), String> {
        if self.connection.lock().unwrap().writer.is_some() {
            return Ok(());
        }

        self.stopping.store(false, Ordering::SeqCst);
        let profile = self.profile();
        if profile.is_configured() {
            self.begin_connecting_ssh();
        }
        else {
            self.set_connection_state(CoreConnectionState::Handshaking);
        }

        let (transport, stream) = match open_transport_stream(&profile, &self.socket_path, &self.stopping) {
            Ok(result) => result,
            Err(error) => {
                self.set_connection_failed(error.clone());
                return Err(error);
            }
        };
        if let Some(ref transport_ref) = transport {
            transport_ref.set_timeout_ms(REMOTE_HANDSHAKE_TIMEOUT.as_millis() as u32);
        }
        self.connection.lock().unwrap().transport = transport;

        if let Err(error) = self.connect_core_stream(stream) {
            if let Some(mut transport) = self.connection.lock().unwrap().transport.take() {
                transport.stop();
            }
            return Err(error);
        }
        Ok(())
    }

    pub fn probe(&self) -> Result<(), String> {
        if self.is_connected() {
            return Ok(());
        }

        self.stopping.store(false, Ordering::SeqCst);
        let profile = self.profile();
        if profile.is_configured() {
            self.begin_connecting_ssh();
        }
        else {
            self.set_connection_state(CoreConnectionState::Handshaking);
        }

        let (mut transport, mut stream) = match open_transport_stream(&profile, &self.socket_path, &self.stopping)
        {
            Ok(result) => result,
            Err(error) => {
                self.set_connection_failed(error.clone());
                return Err(error);
            }
        };
        if let Some(ref transport_ref) = transport {
            transport_ref.set_timeout_ms(REMOTE_HANDSHAKE_TIMEOUT.as_millis() as u32);
        }

        let mut reader = match stream.try_clone() {
            Ok(reader) => reader,
            Err(error) => {
                if let Some(transport) = transport.as_mut() {
                    transport.stop();
                }
                self.set_connection_failed(error.clone());
                return Err(error);
            }
        };
        let _ = reader.set_unix_read_timeout(Some(REMOTE_HANDSHAKE_TIMEOUT));

        let handshake_result = perform_handshake(&mut stream, &mut reader);
        let _ = write_message(&mut stream, &ClientMessage::Disconnect);
        if let Some(transport) = transport.as_mut() {
            transport.stop();
        }

        match handshake_result {
            Ok(()) => {
                self.set_connection_state(CoreConnectionState::Disconnected);
                Ok(())
            }
            Err(error) => {
                self.set_connection_failed(error.clone());
                Err(error)
            }
        }
    }

    pub fn cancel_connect(&self) {
        self.stopping.store(true, Ordering::SeqCst);
    }

    pub fn connect_stream(&self, stream: UnixStream) -> Result<(), String> {
        self.connect_core_stream(CoreClientStream::Unix(stream))
    }

    fn connect_core_stream(&self, mut stream: CoreClientStream) -> Result<(), String> {
        let frontend_update_sender = {
            let connection = match self.connection.lock() {
                Ok(connection) => connection,
                Err(error) => {
                    let message = error.to_string();
                    self.set_connection_failed(message.clone());
                    return Err(message);
                }
            };
            if connection.writer.is_some() {
                return Ok(());
            }
            match connection.frontend_update_sender.clone() {
                Some(sender) => sender,
                None => {
                    let message = String::from("Missing UI update sender");
                    self.set_connection_failed(message.clone());
                    return Err(message);
                }
            }
        };

        self.set_connection_state(CoreConnectionState::Handshaking);

        if let Some(transport) = self.connection.lock().unwrap().transport.as_ref() {
            transport.set_timeout_ms(REMOTE_HANDSHAKE_TIMEOUT.as_millis() as u32);
        }

        let mut reader = match stream.try_clone() {
            Ok(reader) => reader,
            Err(error) => {
                let message = error.to_string();
                self.set_connection_failed(message.clone());
                return Err(message);
            }
        };
        if let Err(error) = reader.set_unix_read_timeout(Some(REMOTE_HANDSHAKE_TIMEOUT)) {
            self.set_connection_failed(error.clone());
            return Err(error);
        }

        if let Err(error) = perform_handshake(&mut stream, &mut reader) {
            self.set_connection_failed(error.clone());
            return Err(error);
        }

        if let Some(transport) = self.connection.lock().unwrap().transport.as_ref() {
            transport.set_timeout_ms(REMOTE_READ_TIMEOUT.as_millis() as u32);
        }
        if let Err(error) = reader.set_unix_read_timeout(Some(REMOTE_READ_TIMEOUT)) {
            let message = error;
            self.set_connection_failed(message.clone());
            return Err(message);
        }

        let writer = Arc::new(Mutex::new(stream));
        let pending_rpc = self.pending_rpc.clone();
        let connection = self.connection.clone();
        let stopping = self.stopping.clone();
        let status = self.status.clone();
        let (stop_sender, stop_receiver) = mpsc::channel();

        self.stopping.store(false, Ordering::SeqCst);

        let response_thread = thread::spawn(move || {
            let disconnect = |message: &str| {
                ::log::error!("{}", message);
                fail_all_pending_rpcs(&pending_rpc, RemoteErrorCode::Internal, message);
                let transport = if let Ok(mut conn) = connection.lock() {
                    conn.writer = None;
                    conn.stop_sender = None;
                    conn.transport.take()
                }
                else {
                    None
                };
                drop(transport);
                if !stopping.load(Ordering::SeqCst) {
                    if let Ok(mut status) = status.lock() {
                        status.set_failed(message);
                    }
                    let _ = frontend_update_sender.send(frontend::UIUpdate::FatalError());
                }
            };

            loop {
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
                        disconnect(&format!("Receive failed: {}", error));
                        return;
                    }
                };

                match message {
                    ServerMessage::Connect { protocol_version } => {
                        ::log::warn!(
                            "Ignoring unexpected Connect after handshake (protocol {})",
                            protocol_version,
                        );
                    }
                    ServerMessage::ExecuteCommand { request_id, invocation_id } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::ExecuteCommand, || {
                            PendingRpcReply::ExecuteCommand(invocation_id)
                        });
                    }
                    ServerMessage::CommandsForHost { request_id, host_id: _, commands } => {
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
                    ServerMessage::CustomCommandsForHost { request_id, host_id: _, commands } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::CustomCommandsForHost, || {
                            PendingRpcReply::CustomCommandsForHost(commands)
                        });
                    }
                    ServerMessage::AllHostCategories { request_id, host_id: _, categories } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::AllHostCategories, || {
                            PendingRpcReply::AllHostCategories(categories)
                        });
                    }
                    ServerMessage::InitialState(display_data) => {
                        for host_display_data in display_data.hosts.into_values() {
                            if frontend_update_sender.send(frontend::UIUpdate::Host(host_display_data)).is_err() {
                                disconnect("Failed to deliver initial state update");
                                return;
                            }
                        }
                    }
                    ServerMessage::HostUpdate(host_display_data) => {
                        if frontend_update_sender.send(frontend::UIUpdate::Host(host_display_data)).is_err() {
                            disconnect("Failed to deliver host update");
                            return;
                        }
                    }
                    ServerMessage::RefreshInvocationIds { request_id, invocation_ids } => {
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
                    ServerMessage::WriteCachedFileResult { request_id } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::WriteCachedFile, || {
                            PendingRpcReply::FileOpDone
                        });
                    }
                    ServerMessage::RemoveCachedFileResult { request_id } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::RemoveCachedFile, || {
                            PendingRpcReply::FileOpDone
                        });
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
                    ServerMessage::Config {
                        request_id,
                        main_yml,
                        hosts_yml,
                        groups_yml,
                    } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::Config, || PendingRpcReply::Config {
                            main_yml,
                            hosts_yml,
                            groups_yml,
                        });
                    }
                    ServerMessage::UpdateConfigOk { request_id } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::UpdateConfig, || {
                            PendingRpcReply::UpdateConfigOk
                        });
                    }
                    ServerMessage::GetSecretResult { request_id, value } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::GetSecret, || {
                            PendingRpcReply::GetSecret(value)
                        });
                    }
                    ServerMessage::StoreSecretResult { request_id, placeholder } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::StoreSecret, || {
                            PendingRpcReply::StoreSecret(placeholder)
                        });
                    }
                    ServerMessage::RemoveSecretResult { request_id } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::RemoveSecret, || {
                            PendingRpcReply::RemoveSecretOk
                        });
                    }
                    ServerMessage::Ack { request_id } => {
                        deliver_response(&pending_rpc, request_id, PendingRpcKind::Ack, || PendingRpcReply::Ack);
                    }
                    ServerMessage::Error {
                        request_id: Some(request_id),
                        code,
                        message,
                    } => {
                        ::log::error!("Core server error [{}]: {}", code, message);
                        match pending_rpc.lock() {
                            Ok(mut map) => {
                                if let Some(pending) = map.remove(&request_id) {
                                    if pending
                                        .sender
                                        .send(PendingRpcReply::Error { code, message })
                                        .is_err()
                                    {
                                        ::log::error!("Receiver dropped");
                                    }
                                }
                                else {
                                    ::log::error!("Received error for unknown request id {}", request_id);
                                }
                            }
                            Err(err) => {
                                ::log::error!("Request failed: {}", err);
                            }
                        }
                    }
                    ServerMessage::Error {
                        request_id: None,
                        code,
                        message,
                    } => {
                        disconnect(&format!("Core server error [{}]: {}", code, message));
                        return;
                    }
                }
            }
        });

        let mut connection = self.connection.lock().unwrap();
        connection.writer = Some(writer);
        connection.stop_sender = Some(stop_sender);
        connection.response_thread = Some(response_thread);
        drop(connection);
        self.set_connection_state(CoreConnectionState::Connected);
        Ok(())
    }

    fn send_nowait(&self, message: &ClientMessage) -> Result<(), String> {
        let writer_hold = self.connection.lock().unwrap().writer.clone();
        let writer = writer_hold.ok_or_else(|| String::from("Not connected to remote core"))?;
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
                    let _ = pending.sender.send(PendingRpcReply::Error {
                        code: RemoteErrorCode::Internal,
                        message: error.clone(),
                    });
                }
            }
            return Err(LkError::new(ErrorKind::ConnectionFailed, error));
        }

        match receiver.recv_timeout(REMOTE_COMMAND_TIMEOUT) {
            Ok(PendingRpcReply::Error { code, message }) => {
                Err(LkError::other(format!("{}: {}", code, message)))
            }
            Ok(reply) => {
                if reply_matches(&kind, &reply) {
                    Ok(reply)
                }
                else {
                    Err(LkError::other("Unexpected response from remote core"))
                }
            }
            Err(recv_error) => {
                if let Ok(mut map) = self.pending_rpc.lock() {
                    let _ = map.remove(&request_id);
                }
                Err(LkError::new(ErrorKind::ConnectionFailed, recv_error.to_string()))
            }
        }
    }

    pub fn stop_connection(&self) {
        self.stopping.store(true, Ordering::SeqCst);
        fail_all_pending_rpcs(&self.pending_rpc, RemoteErrorCode::Internal, "Disconnected from remote core");

        let (writer_opt, stop_sender_opt, thread_opt, transport_opt) = {
            let mut conn = self.connection.lock().unwrap();
            (
                conn.writer.take(),
                conn.stop_sender.take(),
                conn.response_thread.take(),
                conn.transport.take(),
            )
        };

        if let Some(stop_sender) = stop_sender_opt {
            let _ = stop_sender.send(());
        }

        if let Some(writer) = writer_opt {
            let disconnect = ClientMessage::Disconnect;
            if let Ok(mut writer) = writer.lock() {
                let _ = write_message(&mut *writer, &disconnect);
            }
        }

        if let Some(response_thread) = thread_opt {
            if let Err(error) = response_thread.join() {
                ::log::error!("Response thread failed: {:?}", error);
            }
        }

        if let Some(mut transport) = transport_opt {
            transport.stop();
        }

        self.set_connection_state(CoreConnectionState::Disconnected);
    }

    pub fn is_connected(&self) -> bool {
        self.connection.lock().unwrap().writer.is_some()
    }

    pub fn begin_connecting_ssh(&self) {
        self.set_connection_state(CoreConnectionState::ConnectingSsh);
    }

    pub fn begin_reconnecting(&self) {
        self.set_connection_state(CoreConnectionState::Reconnecting);
    }
}

pub struct RemoteCommandBackend {
    client: Arc<RemoteCoreClient>,
}

impl RemoteCommandBackend {
    pub fn new(client: Arc<RemoteCoreClient>) -> Self {
        RemoteCommandBackend { client }
    }

    fn connect(&mut self) -> Result<(), String> {
        self.client.connect()
    }

    pub fn connect_stream(&mut self, stream: UnixStream) -> Result<(), String> {
        self.client.connect_stream(stream)
    }

    pub fn connect_with_frontend_stream(
        &mut self,
        frontend_update_sender: mpsc::Sender<frontend::UIUpdate>,
        stream: UnixStream,
    ) -> Result<(), String> {
        self.client.set_frontend_update_sender(frontend_update_sender);
        self.client.connect_stream(stream)
    }

    fn stop_connection(&mut self) {
        self.client.stop_connection();
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
        self.client.set_frontend_update_sender(frontend_update_sender);
        if let Err(error) = self.connect() {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn start_processing_responses(&mut self) {}

    fn stop(&mut self) {
        self.client.stop_connection();
    }

    fn refresh_host_monitors(&mut self, host_id: &str) {
        if let Err(error) = self.client.send_message_result(PendingRpcKind::Ack, |request_id| {
            ClientMessage::RefreshHostMonitors {
                request_id,
                host_id: host_id.to_string(),
            }
        }) {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn commands_for_host(&self, host_id: &str) -> Result<HashMap<String, CommandButtonData>, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::CommandsForHost, |request_id| ClientMessage::CommandsForHost {
                request_id,
                host_id: host_id.to_string(),
            })? {
            PendingRpcReply::CommandsForHost(commands) => Ok(commands),
            _ => Err(LkError::unexpected()),
        }
    }

    fn command_for_host(&self, host_id: &str, command_id: &str) -> Result<Option<CommandButtonData>, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::CommandForHost, |request_id| ClientMessage::CommandForHost {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
            })? {
            PendingRpcReply::CommandForHost(command) => Ok(command),
            _ => Err(LkError::unexpected()),
        }
    }

    fn custom_commands_for_host(&self, host_id: &str) -> Result<HashMap<String, configuration::CustomCommandConfig>, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::CustomCommandsForHost, |request_id| {
                ClientMessage::CustomCommandsForHost { request_id, host_id: host_id.to_string() }
            })? {
            PendingRpcReply::CustomCommandsForHost(commands) => Ok(commands),
            _ => Err(LkError::unexpected()),
        }
    }

    fn all_host_categories(&self, host_id: &str) -> Result<Vec<String>, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::AllHostCategories, |request_id| ClientMessage::AllHostCategories {
                request_id,
                host_id: host_id.to_string(),
            })? {
            PendingRpcReply::AllHostCategories(categories) => Ok(categories),
            _ => Err(LkError::unexpected()),
        }
    }

    fn execute_command(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Result<u64, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::ExecuteCommand, |request_id| ClientMessage::ExecuteCommand {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
                parameters: parameters.to_vec(),
            })? {
            PendingRpcReply::ExecuteCommand(id) => Ok(id),
            _ => Err(LkError::unexpected()),
        }
    }

    fn interrupt_invocation(&self, invocation_id: u64) {
        if !self.client.is_connected() {
            return;
        }

        if let Err(error) = self.client.send_message_result(PendingRpcKind::Ack, |request_id| {
            ClientMessage::InterruptInvocation {
                request_id,
                invocation_id,
            }
        }) {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn verify_host_key(&self, host_id: &str, connector_id: &str, key_id: &str) {
        if let Err(error) = self.client.send_message_result(PendingRpcKind::Ack, |request_id| {
            ClientMessage::VerifyHostKey {
                request_id,
                host_id: host_id.to_string(),
                connector_id: connector_id.to_string(),
                key_id: key_id.to_string(),
            }
        }) {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn initialize_host(&mut self, host_id: &str) {
        if let Err(error) = self.client.send_message_result(PendingRpcKind::Ack, |request_id| {
            ClientMessage::RefreshPlatformInfo {
                request_id,
                host_id: host_id.to_string(),
            }
        }) {
            ::log::error!("Request failed: {}", error);
        }
    }

    fn initialize_hosts(&mut self) -> Result<Vec<String>, LkError> {
        match self.client.send_message_result(PendingRpcKind::InitializeHosts, |request_id| {
            ClientMessage::RefreshPlatformInfoAll { request_id }
        })? {
            PendingRpcReply::InitializeHosts(host_ids) => Ok(host_ids),
            _ => Err(LkError::unexpected()),
        }
    }

    fn refresh_monitors_for_command(&mut self, host_id: &str, command_id: &str) -> Result<Vec<u64>, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::RefreshInvocationIds, |request_id| {
                ClientMessage::RefreshMonitorsForCommand {
                    request_id,
                    host_id: host_id.to_string(),
                    command_id: command_id.to_string(),
                }
            })? {
            PendingRpcReply::RefreshInvocationIds(invocation_ids) => Ok(invocation_ids),
            _ => Err(LkError::unexpected()),
        }
    }

    fn refresh_monitors_of_category(&mut self, host_id: &str, category: &str) -> Result<Vec<u64>, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::RefreshInvocationIds, |request_id| {
                ClientMessage::RefreshMonitorsOfCategory {
                    request_id,
                    host_id: host_id.to_string(),
                    category: category.to_string(),
                }
            })? {
            PendingRpcReply::RefreshInvocationIds(invocation_ids) => Ok(invocation_ids),
            _ => Err(LkError::unexpected()),
        }
    }

    fn refresh_certificate_monitors(&mut self) -> Result<Vec<u64>, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::RefreshInvocationIds, |request_id| {
                ClientMessage::RefreshCertificateMonitors { request_id }
            })? {
            PendingRpcReply::RefreshInvocationIds(invocation_ids) => Ok(invocation_ids),
            _ => Err(LkError::unexpected()),
        }
    }

    fn resolve_text_editor_path(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> Result<Option<String>, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::ResolveTextEditorPath, |request_id| {
                ClientMessage::ResolveTextEditorPath {
                    request_id,
                    host_id: host_id.to_string(),
                    command_id: command_id.to_string(),
                    parameters: parameters.to_vec(),
                }
            })? {
            PendingRpcReply::ResolveTextEditorPath(path) => Ok(path),
            _ => Err(LkError::unexpected()),
        }
    }

    fn download_editable_file(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> Result<(u64, String), LkError> {
        let remote_path = remote_file_path.to_string();
        match self
            .client
            .send_message_result(PendingRpcKind::DownloadEditable, |request_id| ClientMessage::DownloadEditableFile {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
                remote_file_path: remote_path.clone(),
            })? {
            PendingRpcReply::DownloadEditable(invocation_id) => Ok((invocation_id, remote_path)),
            _ => Err(LkError::unexpected()),
        }
    }

    fn upload_file(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> Result<u64, LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::UploadFromCache, |request_id| ClientMessage::UploadFileFromCache {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
                remote_file_path: remote_file_path.to_string(),
            })? {
            PendingRpcReply::UploadFromCache(id) => Ok(id),
            _ => Err(LkError::unexpected()),
        }
    }

    fn upload_file_from_cache(&mut self, host_id: &str, command_id: &str, remote_file_path: &str) -> Result<u64, LkError> {
        self.upload_file(host_id, command_id, remote_file_path)
    }

    fn write_cached_file(&mut self, host_id: &str, remote_file_path: &str, new_contents: Vec<u8>) -> Result<(), LkError> {
        let host_id = host_id.to_string();
        let remote_file_path = remote_file_path.to_string();
        self.client
            .send_message_result(PendingRpcKind::WriteCachedFile, move |request_id| ClientMessage::WriteCachedFile {
                request_id,
                host_id,
                remote_file_path,
                contents: new_contents,
            })?;
        Ok(())
    }

    fn remove_cached_file(&mut self, host_id: &str, remote_file_path: &str) -> Result<(), LkError> {
        self.client
            .send_message_result(PendingRpcKind::RemoveCachedFile, |request_id| ClientMessage::RemoveCachedFile {
                request_id,
                host_id: host_id.to_string(),
                remote_file_path: remote_file_path.to_string(),
            })?;
        Ok(())
    }

    fn has_cached_file_changed(&self, host_id: &str, remote_file_path: &str, new_contents: &[u8]) -> Result<bool, LkError> {
        let hex = sha256::hash(new_contents);
        match self
            .client
            .send_message_result(PendingRpcKind::HasCachedFileChanged, |request_id| {
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

impl ConfigBackend for RemoteConfigBackend {
    fn get_config(&self) -> Result<(configuration::Configuration, configuration::Hosts, configuration::Groups), LkError> {
        let (main_yml, hosts_yml, groups_yml) = match self
            .client
            .send_message_result(PendingRpcKind::Config, |request_id| ClientMessage::GetConfig { request_id })?
        {
            PendingRpcReply::Config { main_yml, hosts_yml, groups_yml } => (main_yml, hosts_yml, groups_yml),
            _ => return Err(LkError::unexpected()),
        };
        let main_config: configuration::Configuration = serde_yaml::from_str(&main_yml)?;
        let hosts: configuration::Hosts = serde_yaml::from_str(&hosts_yml)?;
        let groups: configuration::Groups = serde_yaml::from_str(&groups_yml)?;
        Ok((main_config, hosts, groups))
    }

    fn update_config(
        &self,
        main_config: configuration::Configuration,
        hosts: configuration::Hosts,
        groups: configuration::Groups,
    ) -> Result<(), LkError> {
        let main_yml = serde_yaml::to_string(&main_config)?;
        let hosts_yml = serde_yaml::to_string(&hosts)?;
        let groups_yml = serde_yaml::to_string(&groups)?;
        match self
            .client
            .send_message_result(PendingRpcKind::UpdateConfig, |request_id| ClientMessage::UpdateConfig {
                request_id,
                main_yml,
                hosts_yml,
                groups_yml,
            })? {
            PendingRpcReply::UpdateConfigOk => Ok(()),
            _ => Err(LkError::unexpected()),
        }
    }

    fn get_secret(&self, source_id: &str, module_id: &str, setting_key: &str) -> Result<Option<String>, LkError> {
        match self.client.send_message_result(PendingRpcKind::GetSecret, |request_id| ClientMessage::GetSecret {
            request_id,
            source_id: source_id.to_string(),
            module_id: module_id.to_string(),
            setting_key: setting_key.to_string(),
        })? {
            PendingRpcReply::GetSecret(value) => Ok(value),
            _ => Err(LkError::unexpected()),
        }
    }

    fn store_secret(
        &self,
        source_id: &str,
        module_id: &str,
        setting_key: &str,
        secret_value: &str,
    ) -> Result<String, LkError> {
        match self.client.send_message_result(PendingRpcKind::StoreSecret, |request_id| ClientMessage::StoreSecret {
            request_id,
            source_id: source_id.to_string(),
            module_id: module_id.to_string(),
            setting_key: setting_key.to_string(),
            secret_value: secret_value.to_string(),
        })? {
            PendingRpcReply::StoreSecret(placeholder) => Ok(placeholder),
            _ => Err(LkError::unexpected()),
        }
    }

    fn remove_secret(&self, source_id: &str, module_id: &str, setting_key: &str) -> Result<(), LkError> {
        match self
            .client
            .send_message_result(PendingRpcKind::RemoveSecret, |request_id| ClientMessage::RemoveSecret {
                request_id,
                source_id: source_id.to_string(),
                module_id: module_id.to_string(),
                setting_key: setting_key.to_string(),
            })? {
            PendingRpcReply::RemoveSecretOk => Ok(()),
            _ => Err(LkError::unexpected()),
        }
    }
}
