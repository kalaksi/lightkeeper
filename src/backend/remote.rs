/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::io;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use crate::command_handler::CommandButtonData;
use crate::configuration;
use crate::connection_manager::ConnectorRequest;
use crate::frontend;
use crate::host_manager::StateUpdateMessage;
use crate::remote_core::protocol::{
    ClientMessage,
    PROTOCOL_VERSION,
    ServerMessage,
    read_message,
    write_message,
};

use super::api::CommandBackend;

//
// Remote backend implementation
//

const REMOTE_READ_TIMEOUT: Duration = Duration::from_millis(100);
const REMOTE_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const REMOTE_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

enum PendingRpc {
    ExecuteCommand(mpsc::Sender<u64>),
    CommandsForHost(mpsc::Sender<HashMap<String, CommandButtonData>>),
    CommandForHost(mpsc::Sender<Option<CommandButtonData>>),
    CustomCommandsForHost(mpsc::Sender<HashMap<String, configuration::CustomCommandConfig>>),
    AllHostCategories(mpsc::Sender<Vec<String>>),
    RefreshInvocationIds(mpsc::Sender<Vec<u64>>),
    InitializeHosts(mpsc::Sender<Vec<String>>),
    ResolveTextEditorPath(mpsc::Sender<Option<String>>),
    DownloadEditable(mpsc::Sender<u64>),
    UploadEdited(mpsc::Sender<u64>),
}

fn fail_pending_rpc(pending: PendingRpc) {
    match pending {
        PendingRpc::ExecuteCommand(sender) => {
            let _ = sender.send(0);
        }
        PendingRpc::CommandsForHost(sender) => {
            let _ = sender.send(HashMap::new());
        }
        PendingRpc::CommandForHost(sender) => {
            let _ = sender.send(None);
        }
        PendingRpc::CustomCommandsForHost(sender) => {
            let _ = sender.send(HashMap::new());
        }
        PendingRpc::AllHostCategories(sender) => {
            let _ = sender.send(Vec::new());
        }
        PendingRpc::RefreshInvocationIds(sender) => {
            let _ = sender.send(Vec::new());
        }
        PendingRpc::InitializeHosts(sender) => {
            let _ = sender.send(Vec::new());
        }
        PendingRpc::ResolveTextEditorPath(sender) => {
            let _ = sender.send(None);
        }
        PendingRpc::DownloadEditable(sender) => {
            let _ = sender.send(0);
        }
        PendingRpc::UploadEdited(sender) => {
            let _ = sender.send(0);
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

    fn allocate_request_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, Ordering::Relaxed)
    }

    fn connect(&mut self) -> Result<(), String> {
        if self.writer.is_some() {
            return Ok(());
        }

        let frontend_update_sender = self.frontend_update_sender.clone()
            .ok_or_else(|| String::from("Remote backend is missing UI update sender"))?;

        let mut stream = UnixStream::connect(&self.socket_path).map_err(|error| error.to_string())?;
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
        let response_thread = thread::spawn(move || {
            loop {
                match stop_receiver.try_recv() {
                    Ok(()) | Err(mpsc::TryRecvError::Disconnected) => return,
                    Err(mpsc::TryRecvError::Empty) => {}
                }

                let message = match read_message::<ServerMessage, _>(&mut reader) {
                    Ok(message) => message,
                    Err(error)
                        if error.kind() == io::ErrorKind::TimedOut
                            || error.kind() == io::ErrorKind::WouldBlock =>
                    {
                        continue;
                    }
                    Err(error) => {
                        ::log::error!("Remote backend receive failed: {}", error);
                        return;
                    }
                };

                let take_rpc = |request_id: u64| -> Option<PendingRpc> {
                    match pending_rpc.lock() {
                        Ok(mut map) => map.remove(&request_id),
                        Err(error) => {
                            ::log::error!("Remote backend RPC map lock failed: {}", error);
                            None
                        }
                    }
                };

                match message {
                    ServerMessage::Connect { protocol_version } => {
                        if protocol_version != PROTOCOL_VERSION {
                            ::log::error!(
                                "Remote backend protocol mismatch: expected {}, got {}",
                                PROTOCOL_VERSION,
                                protocol_version,
                            );
                        }
                    }
                    ServerMessage::ExecuteCommand {
                        request_id,
                        invocation_id,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::ExecuteCommand(sender)) => {
                            if sender.send(invocation_id).is_err() {
                                ::log::error!("Remote backend invocation ack receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for execute command");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected command ack");
                        }
                    },
                    ServerMessage::CommandsForHost {
                        request_id,
                        host_id: _,
                        commands,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::CommandsForHost(sender)) => {
                            if sender.send(commands).is_err() {
                                ::log::error!("Remote backend commands query receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for commands for host");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected commands response");
                        }
                    },
                    ServerMessage::CommandForHost {
                        request_id,
                        host_id: _,
                        command_id: _,
                        command,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::CommandForHost(sender)) => {
                            if sender.send(command).is_err() {
                                ::log::error!("Remote backend command lookup receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for command for host");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected command lookup response");
                        }
                    },
                    ServerMessage::CustomCommandsForHost {
                        request_id,
                        host_id: _,
                        commands,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::CustomCommandsForHost(sender)) => {
                            if sender.send(commands).is_err() {
                                ::log::error!("Remote backend custom command query receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for custom commands");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected custom command response");
                        }
                    },
                    ServerMessage::AllHostCategories {
                        request_id,
                        host_id: _,
                        categories,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::AllHostCategories(sender)) => {
                            if sender.send(categories).is_err() {
                                ::log::error!("Remote backend category query receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for host categories");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected category response");
                        }
                    },
                    ServerMessage::InitialState(display_data) => {
                        for host_display_data in display_data.hosts.into_values() {
                            if frontend_update_sender
                                .send(frontend::UIUpdate::Host(host_display_data))
                                .is_err()
                            {
                                ::log::error!("Remote backend failed to deliver initial state update");
                                return;
                            }
                        }
                    }
                    ServerMessage::HostUpdate(host_display_data) => {
                        if frontend_update_sender
                            .send(frontend::UIUpdate::Host(host_display_data))
                            .is_err()
                        {
                            ::log::error!("Remote backend failed to deliver host update");
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
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::RefreshInvocationIds(sender)) => {
                            if sender.send(invocation_ids).is_err() {
                                ::log::error!("Remote backend refresh id receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for refresh invocation ids");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected refresh id response");
                        }
                    },
                    ServerMessage::InitializeHostsResult {
                        request_id,
                        host_ids,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::InitializeHosts(sender)) => {
                            if sender.send(host_ids).is_err() {
                                ::log::error!("Remote backend init hosts receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for initialize hosts");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected init hosts response");
                        }
                    },
                    ServerMessage::ResolveTextEditorPath {
                        request_id,
                        path,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::ResolveTextEditorPath(sender)) => {
                            if sender.send(path).is_err() {
                                ::log::error!("Remote backend resolve path receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for resolve text editor path");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected resolve path response");
                        }
                    },
                    ServerMessage::DownloadEditableFileResult {
                        request_id,
                        invocation_id,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::DownloadEditable(sender)) => {
                            if sender.send(invocation_id).is_err() {
                                ::log::error!("Remote backend download receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for download editable");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected download response");
                        }
                    },
                    ServerMessage::UploadEditedFileResult {
                        request_id,
                        invocation_id,
                    } => match take_rpc(request_id) {
                        Some(PendingRpc::UploadEdited(sender)) => {
                            if sender.send(invocation_id).is_err() {
                                ::log::error!("Remote backend upload receiver dropped");
                            }
                        }
                        Some(other) => {
                            ::log::error!("Remote backend RPC type mismatch for upload edited");
                            fail_pending_rpc(other);
                        }
                        None => {
                            ::log::error!("Remote backend received unexpected upload response");
                        }
                    },
                    ServerMessage::Error {
                        request_id,
                        message,
                    } => {
                        ::log::error!("Remote backend server error: {}", message);
                        if let Some(request_id) = request_id {
                            if let Some(pending) = take_rpc(request_id) {
                                fail_pending_rpc(pending);
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

    fn send_message(&self, message: &ClientMessage) -> Result<(), String> {
        let writer = self.writer.as_ref()
            .ok_or_else(|| String::from("Remote backend is not connected"))?;
        let mut writer = writer.lock().map_err(|error| error.to_string())?;
        write_message(&mut *writer, message).map_err(|error| error.to_string())
    }

    fn send_message_with_response<Response: Send + 'static>(
        &self,
        register: impl FnOnce(mpsc::Sender<Response>) -> PendingRpc,
        build_message: impl FnOnce(u64) -> ClientMessage,
    ) -> Result<Response, String> {
        let request_id = self.allocate_request_id();
        let (sender, receiver) = mpsc::channel();
        match self.pending_rpc.lock() {
            Ok(mut map) => {
                map.insert(request_id, register(sender));
            }
            Err(error) => return Err(error.to_string()),
        }

        let message = build_message(request_id);
        if let Err(error) = self.send_message(&message) {
            if let Ok(mut map) = self.pending_rpc.lock() {
                if let Some(pending) = map.remove(&request_id) {
                    fail_pending_rpc(pending);
                }
            }
            return Err(error);
        }

        receiver
            .recv_timeout(REMOTE_QUERY_TIMEOUT)
            .map_err(|error| error.to_string())
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
                ::log::error!("Remote backend response thread failed: {:?}", error);
            }
        }

        self.writer = None;
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
    }

    fn start_processing_responses(&mut self) {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
        }
    }

    fn stop(&mut self) {
        self.stop_connection();
    }

    fn refresh_host_monitors(&mut self, host_id: &str) {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return;
        }

        if let Err(error) = self.send_message(&ClientMessage::RefreshHostMonitors {
            host_id: host_id.to_string(),
        }) {
            ::log::error!("Remote backend refresh_host_monitors failed: {}", error);
        }
    }

    fn commands_for_host(&self, host_id: &str) -> HashMap<String, CommandButtonData> {
        match self.send_message_with_response(
            |sender| PendingRpc::CommandsForHost(sender),
            |request_id| ClientMessage::CommandsForHost {
                request_id,
                host_id: host_id.to_string(),
            },
        ) {
            Ok(commands) => commands,
            Err(error) => {
                ::log::error!("Remote backend commands query failed: {}", error);
                HashMap::new()
            }
        }
    }

    fn command_for_host(&self, host_id: &str, command_id: &str) -> Option<CommandButtonData> {
        match self.send_message_with_response(
            |sender| PendingRpc::CommandForHost(sender),
            |request_id| ClientMessage::CommandForHost {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
            },
        ) {
            Ok(command) => command,
            Err(error) => {
                ::log::error!("Remote backend command lookup failed: {}", error);
                None
            }
        }
    }

    fn custom_commands_for_host(
        &self,
        host_id: &str,
    ) -> HashMap<String, configuration::CustomCommandConfig> {
        match self.send_message_with_response(
            |sender| PendingRpc::CustomCommandsForHost(sender),
            |request_id| ClientMessage::CustomCommandsForHost {
                request_id,
                host_id: host_id.to_string(),
            },
        ) {
            Ok(commands) => commands,
            Err(error) => {
                ::log::error!("Remote backend custom commands query failed: {}", error);
                HashMap::new()
            }
        }
    }

    fn all_host_categories(&self, host_id: &str) -> Vec<String> {
        match self.send_message_with_response(
            |sender| PendingRpc::AllHostCategories(sender),
            |request_id| ClientMessage::AllHostCategories {
                request_id,
                host_id: host_id.to_string(),
            },
        ) {
            Ok(categories) => categories,
            Err(error) => {
                ::log::error!("Remote backend category query failed: {}", error);
                Vec::new()
            }
        }
    }

    fn execute_command(&mut self, host_id: &str, command_id: &str, parameters: &[String]) -> u64 {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return 0;
        }

        let request_id = self.allocate_request_id();
        let (sender, receiver) = mpsc::channel();
        match self.pending_rpc.lock() {
            Ok(mut map) => {
                map.insert(request_id, PendingRpc::ExecuteCommand(sender));
            }
            Err(error) => {
                ::log::error!("Remote backend invocation queue failed: {}", error);
                return 0;
            }
        }

        if let Err(error) = self.send_message(&ClientMessage::ExecuteCommand {
            request_id,
            host_id: host_id.to_string(),
            command_id: command_id.to_string(),
            parameters: parameters.to_vec(),
        }) {
            if let Ok(mut map) = self.pending_rpc.lock() {
                if let Some(pending) = map.remove(&request_id) {
                    fail_pending_rpc(pending);
                }
            }
            ::log::error!("Remote backend execute failed: {}", error);
            return 0;
        }

        match receiver.recv_timeout(REMOTE_COMMAND_TIMEOUT) {
            Ok(invocation_id) => invocation_id,
            Err(error) => {
                ::log::error!("Remote backend execute timed out: {}", error);
                0
            }
        }
    }

    fn interrupt_invocation(&self, invocation_id: u64) {
        if self.writer.is_none() {
            return;
        }

        if let Err(error) = self.send_message(&ClientMessage::InterruptInvocation { invocation_id }) {
            ::log::error!("Remote backend interrupt failed: {}", error);
        }
    }

    fn verify_host_key(&self, host_id: &str, connector_id: &str, key_id: &str) {
        if let Err(error) = self.send_message(&ClientMessage::VerifyHostKey {
            host_id: host_id.to_string(),
            connector_id: connector_id.to_string(),
            key_id: key_id.to_string(),
        }) {
            ::log::error!("Remote backend verify failed: {}", error);
        }
    }

    fn initialize_host(&mut self, host_id: &str) {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return;
        }

        if let Err(error) = self.send_message(&ClientMessage::RefreshPlatformInfo {
            host_id: host_id.to_string(),
        }) {
            ::log::error!("Remote backend initialize_host failed: {}", error);
        }
    }

    fn initialize_hosts(&mut self) -> Vec<String> {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return Vec::new();
        }

        match self.send_message_with_response(
            |sender| PendingRpc::InitializeHosts(sender),
            |request_id| ClientMessage::RefreshPlatformInfoAll { request_id },
        ) {
            Ok(host_ids) => host_ids,
            Err(error) => {
                ::log::error!("Remote backend initialize_hosts failed: {}", error);
                Vec::new()
            }
        }
    }

    fn refresh_monitors_for_command(&mut self, host_id: &str, command_id: &str) -> Vec<u64> {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return Vec::new();
        }

        match self.send_message_with_response(
            |sender| PendingRpc::RefreshInvocationIds(sender),
            |request_id| ClientMessage::RefreshMonitorsForCommand {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
            },
        ) {
            Ok(invocation_ids) => invocation_ids,
            Err(error) => {
                ::log::error!("Remote backend refresh_monitors_for_command failed: {}", error);
                Vec::new()
            }
        }
    }

    fn refresh_monitors_of_category(&mut self, host_id: &str, category: &str) -> Vec<u64> {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return Vec::new();
        }

        match self.send_message_with_response(
            |sender| PendingRpc::RefreshInvocationIds(sender),
            |request_id| ClientMessage::RefreshMonitorsOfCategory {
                request_id,
                host_id: host_id.to_string(),
                category: category.to_string(),
            },
        ) {
            Ok(invocation_ids) => invocation_ids,
            Err(error) => {
                ::log::error!("Remote backend refresh_monitors_of_category failed: {}", error);
                Vec::new()
            }
        }
    }

    fn refresh_certificate_monitors(&mut self) -> Vec<u64> {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return Vec::new();
        }

        match self.send_message_with_response(
            |sender| PendingRpc::RefreshInvocationIds(sender),
            |request_id| ClientMessage::RefreshCertificateMonitors { request_id },
        ) {
            Ok(invocation_ids) => invocation_ids,
            Err(error) => {
                ::log::error!("Remote backend refresh_certificate_monitors failed: {}", error);
                Vec::new()
            }
        }
    }

    fn resolve_text_editor_path(
        &mut self,
        host_id: &str,
        command_id: &str,
        parameters: &[String],
    ) -> Option<String> {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return None;
        }

        match self.send_message_with_response(
            |sender| PendingRpc::ResolveTextEditorPath(sender),
            |request_id| ClientMessage::ResolveTextEditorPath {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
                parameters: parameters.to_vec(),
            },
        ) {
            Ok(path) => path,
            Err(error) => {
                ::log::error!("Remote backend resolve_text_editor_path failed: {}", error);
                None
            }
        }
    }

    fn download_editable_file(
        &mut self,
        host_id: &str,
        command_id: &str,
        remote_file_path: &str,
    ) -> (u64, String) {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return (0, String::new());
        }

        let remote_path = remote_file_path.to_string();
        match self.send_message_with_response(
            |sender| PendingRpc::DownloadEditable(sender),
            |request_id| ClientMessage::DownloadEditableFile {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
                remote_file_path: remote_path.clone(),
            },
        ) {
            Ok(invocation_id) => (invocation_id, remote_path),
            Err(error) => {
                ::log::error!("Remote backend download_editable_file failed: {}", error);
                (0, String::new())
            }
        }
    }

    fn upload_file(&mut self, _host_id: &str, _command_id: &str, _local_file_path: &str) -> u64 {
        ::log::error!("Remote backend does not support '{}'", "upload_file");
        0
    }

    fn upload_file_from_editor(&mut self, host_id: &str, command_id: &str, remote_file_path: &str, contents: Vec<u8>) -> u64 {
        if let Err(error) = self.connect() {
            ::log::error!("Failed to connect remote backend: {}", error);
            return 0;
        }

        match self.send_message_with_response(
            |sender| PendingRpc::UploadEdited(sender),
            move |request_id| ClientMessage::UploadEditedFile {
                request_id,
                host_id: host_id.to_string(),
                command_id: command_id.to_string(),
                remote_file_path: remote_file_path.to_string(),
                contents,
            },
        ) {
            Ok(invocation_id) => invocation_id,
            Err(error) => {
                ::log::error!("Remote backend upload_file_from_editor failed: {}", error);
                0
            }
        }
    }

    fn write_file(&mut self, _local_file_path: &str, _new_contents: Vec<u8>) {}

    fn remove_file(&mut self, _local_file_path: &str) {}

    fn has_file_changed(&self, _local_file_path: &str, new_contents: &[u8]) -> bool {
        !new_contents.is_empty()
    }
}
