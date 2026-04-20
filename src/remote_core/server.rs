/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::error::LkError;
use crate::remote_core::protocol::{ClientMessage, PROTOCOL_VERSION, ServerMessage, read_message};
use crate::remote_core::runtime::CoreRuntime;
use crate::remote_core::session::RemoteSession;

pub struct CoreServer {
    listener: UnixListener,
    socket_path: PathBuf,
    runtime: CoreRuntime,
    client_session_active: Arc<Mutex<bool>>,
}

impl CoreServer {
    pub fn start(socket_path: PathBuf, runtime: CoreRuntime) -> Result<(), LkError> {
        if let Some(parent_dir) = socket_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        Self::remove_stale_socket(&socket_path)?;
        let listener = UnixListener::bind(&socket_path)?;
        let mut server = CoreServer {
            listener,
            socket_path,
            runtime,
            client_session_active: Arc::new(Mutex::new(false)),
        };

        server.run()
    }

    fn run(&mut self) -> Result<(), LkError> {
        log::info!("Listening on {}", self.socket_path.display());

        loop {
            let (stream, _address) = self.listener.accept()?;

            if let Err(error) = self.handle_client(stream) {
                log::error!("Client session failed: {}", error);
            }
        }
    }

    fn handle_client(&mut self, mut stream: UnixStream) -> Result<(), LkError> {
        let protocol_version = match read_message::<ClientMessage, _>(&mut stream) {
            Ok(ClientMessage::Connect { protocol_version }) => protocol_version,
            Ok(_) => {
                let session = RemoteSession::new(stream.try_clone()?);
                session.send_error("Expected connect as the first message")?;
                return Ok(());
            }
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error.into()),
        };

        if protocol_version != PROTOCOL_VERSION {
            let session = RemoteSession::new(stream.try_clone()?);
            session.send_error(
                format!(
                    "Unsupported protocol version {}. Expected {}.",
                    protocol_version,
                    PROTOCOL_VERSION,
                ),
            )?;
            return Ok(());
        }

        {
            let mut active = self.client_session_active.lock().unwrap();
            if *active {
                let session = RemoteSession::new(stream.try_clone()?);
                session.send_message(&ServerMessage::Connect {
                    protocol_version: PROTOCOL_VERSION,
                })?;
                session.send_error("Another desktop client is already connected")?;
                return Ok(());
            }
            *active = true;
        }

        struct ClearClientSessionFlag {
            flag: Arc<Mutex<bool>>,
        }

        impl Drop for ClearClientSessionFlag {
            fn drop(&mut self) {
                if let Ok(mut active) = self.flag.lock() {
                    *active = false;
                }
            }
        }

        let _clear_session = ClearClientSessionFlag {
            flag: self.client_session_active.clone(),
        };

        self.handle_connected_client(stream)
    }

    fn handle_connected_client(&mut self, mut stream: UnixStream) -> Result<(), LkError> {
        let update_receiver = self.runtime.new_update_receiver();
        let mut session = RemoteSession::new(stream.try_clone()?);
        session.send_message(&ServerMessage::Connect {
            protocol_version: PROTOCOL_VERSION,
        })?;
        session.send_message(&ServerMessage::InitialState(self.runtime.display_data()))?;
        session.start_update_stream(update_receiver);

        loop {
            let message = match read_message::<ClientMessage, _>(&mut stream) {
                Ok(message) => message,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
                Err(error) => return Err(error.into()),
            };

            match message {
                ClientMessage::Disconnect => return Ok(()),
                ClientMessage::Connect { .. } => {
                    session.send_error("Connect may only be sent once")?;
                }
                ClientMessage::ExecuteCommand {
                    request_id,
                    host_id,
                    command_id,
                    parameters,
                } => {
                    let invocation_id = self.runtime.execute_command(&host_id, &command_id, &parameters);
                    session.send_message(&ServerMessage::ExecuteCommand {
                        request_id,
                        invocation_id,
                    })?;
                }
                ClientMessage::CommandsForHost { request_id, host_id } => {
                    session.send_message(&ServerMessage::CommandsForHost {
                        request_id,
                        host_id: host_id.clone(),
                        commands: self.runtime.commands_for_host(&host_id),
                    })?;
                }
                ClientMessage::CommandForHost {
                    request_id,
                    host_id,
                    command_id,
                } => {
                    session.send_message(&ServerMessage::CommandForHost {
                        request_id,
                        host_id: host_id.clone(),
                        command_id: command_id.clone(),
                        command: self.runtime.command_for_host(&host_id, &command_id),
                    })?;
                }
                ClientMessage::CustomCommandsForHost { request_id, host_id } => {
                    session.send_message(&ServerMessage::CustomCommandsForHost {
                        request_id,
                        host_id: host_id.clone(),
                        commands: self.runtime.custom_commands_for_host(&host_id),
                    })?;
                }
                ClientMessage::AllHostCategories { request_id, host_id } => {
                    session.send_message(&ServerMessage::AllHostCategories {
                        request_id,
                        host_id: host_id.clone(),
                        categories: self.runtime.all_host_categories(&host_id),
                    })?;
                }
                ClientMessage::VerifyHostKey { host_id, connector_id, key_id } => {
                    self.runtime.verify_host_key(&host_id, &connector_id, &key_id);
                }
                ClientMessage::InterruptInvocation { invocation_id } => {
                    self.runtime.interrupt_invocation(invocation_id);
                }
                ClientMessage::RefreshHostMonitors { host_id } => {
                    self.runtime.refresh_host_monitors(&host_id);
                }
                ClientMessage::RefreshPlatformInfo { host_id } => {
                    self.runtime.refresh_platform_info(&host_id);
                }
                ClientMessage::RefreshPlatformInfoAll { request_id } => {
                    let host_ids = self.runtime.refresh_platform_info_all();
                    session.send_message(&ServerMessage::InitializeHostsResult {
                        request_id,
                        host_ids,
                    })?;
                }
                ClientMessage::RefreshMonitorsForCommand {
                    request_id,
                    host_id,
                    command_id,
                } => {
                    let invocation_ids = self.runtime.refresh_monitors_for_command(&host_id, &command_id);
                    session.send_message(&ServerMessage::RefreshInvocationIds {
                        request_id,
                        invocation_ids,
                    })?;
                }
                ClientMessage::RefreshMonitorsOfCategory {
                    request_id,
                    host_id,
                    category,
                } => {
                    let invocation_ids = self.runtime.refresh_monitors_of_category(&host_id, &category);
                    session.send_message(&ServerMessage::RefreshInvocationIds {
                        request_id,
                        invocation_ids,
                    })?;
                }
                ClientMessage::RefreshCertificateMonitors { request_id } => {
                    let invocation_ids = self.runtime.refresh_certificate_monitors();
                    session.send_message(&ServerMessage::RefreshInvocationIds {
                        request_id,
                        invocation_ids,
                    })?;
                }
                ClientMessage::ResolveTextEditorPath {
                    request_id,
                    host_id,
                    command_id,
                    parameters,
                } => {
                    let path = self.runtime.resolve_text_editor_path(&host_id, &command_id, &parameters);
                    session.send_message(&ServerMessage::ResolveTextEditorPath {
                        request_id,
                        path,
                    })?;
                }
                ClientMessage::DownloadEditableFile {
                    request_id,
                    host_id,
                    command_id,
                    remote_file_path,
                } => {
                    let invocation_id = self.runtime.download_editable_file(
                        &host_id,
                        &command_id,
                        &remote_file_path,
                    );
                    session.send_message(&ServerMessage::DownloadEditableFileResult {
                        request_id,
                        invocation_id,
                    })?;
                }
                ClientMessage::UploadEditedFile {
                    request_id,
                    host_id,
                    command_id,
                    remote_file_path,
                    contents,
                } => {
                    let invocation_id = self.runtime.upload_edited_file(
                        &host_id,
                        &command_id,
                        &remote_file_path,
                        contents,
                    );
                    session.send_message(&ServerMessage::UploadEditedFileResult {
                        request_id,
                        invocation_id,
                    })?;
                }
            }
        }
    }

    fn remove_stale_socket(socket_path: &Path) -> io::Result<()> {
        match fs::remove_file(socket_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

impl Drop for CoreServer {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.socket_path) {
            if error.kind() != io::ErrorKind::NotFound {
                log::warn!("Failed to remove socket {}: {}", self.socket_path.display(), error);
            }
        }
    }
}
