/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::io;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::configuration::{Configuration, Groups, Hosts};
use crate::error::{ErrorKind, LkError};
use crate::remote_core::protocol::{read_message, ClientMessage, RemoteErrorCode, ServerMessage, PROTOCOL_VERSION};
use crate::remote_core::runtime::CoreRuntime;
use crate::remote_core::session::RemoteSession;
use crate::remote_core::socket;
use crate::secrets_manager;

const HANDSHAKE_READ_TIMEOUT: Duration = Duration::from_secs(5);

enum SessionClaim {
    Claim,
    AlreadyClaimed,
}

/// Clears the accept-thread session claim when dropped, if armed.
/// Armed immediately for streams already claimed by the accept thread so failed
/// handshakes still release Busy; armed after a successful in-band claim otherwise.
struct SessionClaimGuard {
    flag: Arc<Mutex<bool>>,
    armed: bool,
}

impl Drop for SessionClaimGuard {
    fn drop(&mut self) {
        if self.armed {
            if let Ok(mut active) = self.flag.lock() {
                *active = false;
            }
        }
    }
}

pub fn run_remote_client_session(
    stream: UnixStream,
    runtime: &mut CoreRuntime,
    client_session_active: &Arc<Mutex<bool>>,
) -> Result<(), LkError> {
    run_remote_client_session_with_claim(stream, runtime, client_session_active, SessionClaim::Claim)
}

/// Runs a session for a stream whose session claim was already taken by the accept thread.
pub fn run_claimed_remote_client_session(
    stream: UnixStream,
    runtime: &mut CoreRuntime,
    client_session_active: &Arc<Mutex<bool>>,
) -> Result<(), LkError> {
    run_remote_client_session_with_claim(stream, runtime, client_session_active, SessionClaim::AlreadyClaimed)
}

fn run_remote_client_session_with_claim(
    mut stream: UnixStream,
    runtime: &mut CoreRuntime,
    client_session_active: &Arc<Mutex<bool>>,
    claim: SessionClaim,
) -> Result<(), LkError> {
    let mut claim_guard = SessionClaimGuard {
        flag: client_session_active.clone(),
        armed: matches!(claim, SessionClaim::AlreadyClaimed),
    };

    stream.set_read_timeout(Some(HANDSHAKE_READ_TIMEOUT))?;

    let protocol_version = match read_message::<ClientMessage, _>(&mut stream) {
        Ok(ClientMessage::Connect { protocol_version }) => protocol_version,
        Ok(_) => {
            let session = RemoteSession::new(stream.try_clone()?);
            session.send_error(RemoteErrorCode::InvalidRequest, "Expected connect as the first message")?;
            return Ok(());
        }
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::TimedOut || error.kind() == io::ErrorKind::WouldBlock => {
            return Err(LkError::other("Client handshake timed out"));
        }
        Err(error) => return Err(error.into()),
    };

    if protocol_version != PROTOCOL_VERSION {
        let session = RemoteSession::new(stream.try_clone()?);
        session.send_error(
            RemoteErrorCode::UnsupportedVersion,
            format!(
                "Unsupported protocol version {}. Expected {}.",
                protocol_version, PROTOCOL_VERSION,
            ),
        )?;
        return Ok(());
    }

    match claim {
        SessionClaim::Claim => {
            let mut active = client_session_active.lock().unwrap();
            if *active {
                let session = RemoteSession::new(stream.try_clone()?);
                session.send_error(RemoteErrorCode::Busy, "Another desktop client is already connected")?;
                return Ok(());
            }
            *active = true;
            claim_guard.armed = true;
        }
        SessionClaim::AlreadyClaimed => {}
    }

    stream.set_read_timeout(None)?;
    handle_connected_client_loop(&mut stream, runtime)
}

fn reject_busy_client(stream: UnixStream) {
    if let Ok(clone) = stream.try_clone() {
        let session = RemoteSession::new(clone);
        let _ = session.send_error(RemoteErrorCode::Busy, "Another desktop client is already connected");
    }
}

fn handle_connected_client_loop(stream: &mut UnixStream, runtime: &mut CoreRuntime) -> Result<(), LkError> {
    let update_receiver = runtime.new_update_receiver();
    let mut session = RemoteSession::new(stream.try_clone()?);
    session.send_message(&ServerMessage::Connect { protocol_version: PROTOCOL_VERSION })?;

    let display_data = runtime.core.host_manager.borrow().get_display_data();
    session.send_message(&ServerMessage::InitialState(display_data))?;
    session.start_update_stream(update_receiver);

    loop {
        let message = match read_message::<ClientMessage, _>(stream) {
            Ok(message) => message,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error.into()),
        };

        match message {
            ClientMessage::Disconnect => return Ok(()),
            ClientMessage::Connect { .. } => {
                session.send_error(RemoteErrorCode::InvalidRequest, "Connect may only be sent once")?;
            }
            ClientMessage::ExecuteCommand {
                request_id,
                host_id,
                command_id,
                parameters,
            } => match runtime.core.command_handler.execute(&host_id, &command_id, &parameters) {
                Ok(invocation_id) => {
                    session.send_message(&ServerMessage::ExecuteCommand { request_id, invocation_id })?;
                }
                Err(error) => {
                    session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                }
            },
            ClientMessage::CommandsForHost { request_id, host_id } => {
                session.send_message(&ServerMessage::CommandsForHost {
                    request_id,
                    host_id: host_id.clone(),
                    commands: runtime.core.command_handler.get_commands_for_host(host_id.clone()),
                })?;
            }
            ClientMessage::CommandForHost { request_id, host_id, command_id } => {
                session.send_message(&ServerMessage::CommandForHost {
                    request_id,
                    host_id: host_id.clone(),
                    command_id: command_id.clone(),
                    command: runtime.core.command_handler.get_command_for_host(&host_id, &command_id),
                })?;
            }
            ClientMessage::CustomCommandsForHost { request_id, host_id } => {
                session.send_message(&ServerMessage::CustomCommandsForHost {
                    request_id,
                    host_id: host_id.clone(),
                    commands: runtime.core.command_handler.get_custom_commands_for_host(&host_id),
                })?;
            }
            ClientMessage::AllHostCategories { request_id, host_id } => {
                session.send_message(&ServerMessage::AllHostCategories {
                    request_id,
                    host_id: host_id.clone(),
                    categories: runtime.core.monitor_manager.get_all_host_categories(&host_id),
                })?;
            }
            ClientMessage::VerifyHostKey {
                request_id,
                host_id,
                connector_id,
                key_id,
            } => {
                runtime.core.command_handler.verify_host_key(&host_id, &connector_id, &key_id);
                session.send_message(&ServerMessage::Ack { request_id })?;
            }
            ClientMessage::InterruptInvocation { request_id, invocation_id } => {
                runtime.core.command_handler.interrupt_invocation(invocation_id);
                session.send_message(&ServerMessage::Ack { request_id })?;
            }
            ClientMessage::RefreshHostMonitors { request_id, host_id } => {
                for category in runtime.core.monitor_manager.get_all_host_categories(&host_id) {
                    let _invocation_ids = runtime.core.monitor_manager.refresh_monitors_of_category(&host_id, &category);
                }
                session.send_message(&ServerMessage::Ack { request_id })?;
            }
            ClientMessage::RefreshPlatformInfo { request_id, host_id } => {
                runtime.core.monitor_manager.refresh_platform_info(&host_id);
                session.send_message(&ServerMessage::Ack { request_id })?;
            }
            ClientMessage::RefreshPlatformInfoAll { request_id } => {
                let host_ids = runtime.core.monitor_manager.refresh_platform_info_all();
                session.send_message(&ServerMessage::InitializeHostsResult { request_id, host_ids })?;
            }
            ClientMessage::RefreshMonitorsForCommand { request_id, host_id, command_id } => {
                let invocation_ids = match runtime.core.command_handler.get_command_for_host(&host_id, &command_id) {
                    None => Vec::new(),
                    Some(command) => {
                        if command.display_options.parent_id.is_empty() {
                            runtime
                                .core
                                .monitor_manager
                                .refresh_monitors_of_category(&host_id, &command.display_options.category)
                        }
                        else {
                            runtime
                                .core
                                .monitor_manager
                                .refresh_monitors_by_id(&host_id, &command.display_options.parent_id)
                        }
                    }
                };
                session.send_message(&ServerMessage::RefreshInvocationIds { request_id, invocation_ids })?;
            }
            ClientMessage::RefreshMonitorsOfCategory { request_id, host_id, category } => {
                let invocation_ids = runtime.core.monitor_manager.refresh_monitors_of_category(&host_id, &category);
                session.send_message(&ServerMessage::RefreshInvocationIds { request_id, invocation_ids })?;
            }
            ClientMessage::RefreshCertificateMonitors { request_id } => {
                let invocation_ids = runtime.core.monitor_manager.refresh_certificate_monitors();
                session.send_message(&ServerMessage::RefreshInvocationIds { request_id, invocation_ids })?;
            }
            ClientMessage::ResolveTextEditorPath {
                request_id,
                host_id,
                command_id,
                parameters,
            } => {
                let path = if let Some(path) = parameters.first().cloned() {
                    Some(path)
                }
                else {
                    runtime.core.command_handler.get_connector_message(&host_id, &command_id)
                };
                session.send_message(&ServerMessage::ResolveTextEditorPath { request_id, path })?;
            }
            ClientMessage::DownloadEditableFile {
                request_id,
                host_id,
                command_id,
                remote_file_path,
            } => {
                match runtime
                    .core
                    .command_handler
                    .download_editable_file(&host_id, &command_id, &remote_file_path)
                {
                    Ok((invocation_id, _)) => {
                        session.send_message(&ServerMessage::DownloadEditableFileResult { request_id, invocation_id })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
            ClientMessage::WriteCachedFile {
                request_id,
                host_id,
                remote_file_path,
                contents,
            } => {
                match runtime.core.command_handler.cache_file_path_for_remote(&host_id, &remote_file_path)
                    .and_then(|path| runtime.core.command_handler.write_file(&path, contents))
                {
                    Ok(()) => {
                        session.send_message(&ServerMessage::WriteCachedFileResult { request_id })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
            ClientMessage::RemoveCachedFile { request_id, host_id, remote_file_path } => {
                match runtime.core.command_handler.cache_file_path_for_remote(&host_id, &remote_file_path)
                    .and_then(|path| runtime.core.command_handler.remove_file(&path))
                {
                    Ok(()) => {
                        session.send_message(&ServerMessage::RemoveCachedFileResult { request_id })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
            ClientMessage::HasCachedFileChanged {
                request_id,
                host_id,
                remote_file_path,
                content_hash,
            } => {
                match runtime
                    .core
                    .command_handler
                    .has_file_changed(&host_id, &remote_file_path, &content_hash)
                {
                    Ok(changed) => {
                        session.send_message(&ServerMessage::HasCachedFileChangedResult { request_id, changed })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
            ClientMessage::UploadFileFromCache {
                request_id,
                host_id,
                command_id,
                remote_file_path,
            } => {
                match runtime.core.command_handler.cache_file_path_for_remote(&host_id, &remote_file_path)
                    .and_then(|path| runtime.core.command_handler.upload_file(&host_id, &command_id, &path))
                {
                    Ok(invocation_id) => {
                        session.send_message(&ServerMessage::UploadFileFromCacheResult { request_id, invocation_id })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
            ClientMessage::GetConfig { request_id } => {
                let result: Result<(String, String, String), LkError> = (|| {
                    let (main, hosts, groups) = Configuration::read(&runtime.config_dir)?;
                    let main_yml = serde_yaml::to_string(&main)?;
                    let hosts_yml = serde_yaml::to_string(&hosts)?;
                    let groups_yml = serde_yaml::to_string(&groups)?;
                    Ok((main_yml, hosts_yml, groups_yml))
                })();
                match result {
                    Ok((main_yml, hosts_yml, groups_yml)) => {
                        session.send_message(&ServerMessage::Config {
                            request_id,
                            main_yml,
                            hosts_yml,
                            groups_yml,
                        })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
            ClientMessage::UpdateConfig {
                request_id,
                main_yml,
                hosts_yml,
                groups_yml,
            } => {
                session.halt_update_stream();

                let parsed = (|| -> Result<(Configuration, Hosts, Groups), LkError> {
                    Ok((
                        serde_yaml::from_str(&main_yml)?,
                        serde_yaml::from_str(&hosts_yml)?,
                        serde_yaml::from_str(&groups_yml)?,
                    ))
                })();

                let (main, hosts, groups) = match parsed {
                    Ok(configs) => configs,
                    Err(error) => {
                        session.start_update_stream(runtime.new_update_receiver());
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                        continue;
                    }
                };

                if let Err(error) =
                    Configuration::write_all_configs_transactional(&runtime.config_dir, &main, &hosts, &groups)
                {
                    session.start_update_stream(runtime.new_update_receiver());
                    session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    continue;
                }

                let module_factory = runtime.core.module_factory.clone();
                runtime.stop();

                let reinit_error = match Configuration::read(&runtime.config_dir) {
                    Ok((main_read, hosts_read, _groups)) => {
                        match crate::initialize_core(&main_read, &hosts_read, module_factory.clone()) {
                            Ok(core) => {
                                runtime.core = core;
                                None
                            }
                            Err(error) => Some(error),
                        }
                    }
                    Err(error) => Some(error.into()),
                };

                if let Some(error) = reinit_error {
                    if let Err(restore_error) = Configuration::restore_config_backups(&runtime.config_dir) {
                        log::error!("Failed to restore configuration backups: {}", restore_error);
                    }
                    let recovered = match Configuration::read(&runtime.config_dir) {
                        Ok((main_read, hosts_read, _groups)) => {
                            match crate::initialize_core(&main_read, &hosts_read, module_factory) {
                                Ok(core) => {
                                    runtime.core = core;
                                    true
                                }
                                Err(recover_error) => {
                                    log::error!("Failed to recover previous core runtime: {}", recover_error);
                                    false
                                }
                            }
                        }
                        Err(read_error) => {
                            log::error!("Failed to read configuration while recovering: {}", read_error);
                            false
                        }
                    };
                    if !recovered {
                        let message = format!(
                            "Unrecoverable core runtime after config update failure: {}",
                            error,
                        );
                        let _ = session.send_request_error(request_id, RemoteErrorCode::Internal, &message);
                        return Err(LkError::new(ErrorKind::Fatal, message));
                    }
                    session.start_update_stream(runtime.new_update_receiver());
                    session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    continue;
                }

                if let Err(error) = Configuration::clear_config_backups(&runtime.config_dir) {
                    log::warn!("Failed to clear configuration backups: {}", error);
                }
                session.send_message(&ServerMessage::InitialState(
                    runtime.core.host_manager.borrow().get_display_data(),
                ))?;
                session.start_update_stream(runtime.new_update_receiver());
                session.send_message(&ServerMessage::UpdateConfigOk { request_id })?;
            }
            ClientMessage::GetSecret {
                request_id,
                source_id,
                module_id,
                setting_key,
            } => {
                let lookup_key = secrets_manager::secret_lookup_key(&module_id, &source_id, &setting_key);
                match secrets_manager::get(&lookup_key) {
                    Ok(value) => {
                        session.send_message(&ServerMessage::GetSecretResult { request_id, value })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
            ClientMessage::StoreSecret {
                request_id,
                source_id,
                module_id,
                setting_key,
                secret_value,
            } => {
                let lookup_key = secrets_manager::secret_lookup_key(&module_id, &source_id, &setting_key);
                match secrets_manager::set(&lookup_key, &secret_value) {
                    Ok(()) => {
                        let placeholder = format!("{}{}", secrets_manager::KEYRING_PREFIX, lookup_key);
                        session.send_message(&ServerMessage::StoreSecretResult {
                            request_id,
                            placeholder,
                        })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
            ClientMessage::RemoveSecret {
                request_id,
                source_id,
                module_id,
                setting_key,
            } => {
                let lookup_key = secrets_manager::secret_lookup_key(&module_id, &source_id, &setting_key);
                match secrets_manager::delete(&lookup_key) {
                    Ok(()) => {
                        session.send_message(&ServerMessage::RemoveSecretResult { request_id })?;
                    }
                    Err(error) => {
                        session.send_request_error(request_id, RemoteErrorCode::Internal, error.to_string())?;
                    }
                }
            }
        }
    }
}

pub struct CoreListener {
    socket_path: PathBuf,
    client_session_active: Arc<Mutex<bool>>,
    incoming: mpsc::Receiver<UnixStream>,
    _accept_thread: thread::JoinHandle<()>,
}

impl CoreListener {
    pub fn bind(socket_path: PathBuf) -> Result<Self, LkError> {
        socket::prepare_socket_path(&socket_path)?;
        let listener = socket::bind_listener(&socket_path)?;

        let (incoming_tx, incoming_rx) = mpsc::channel();
        let client_session_active = Arc::new(Mutex::new(false));
        let accept_active = client_session_active.clone();

        let accept_thread = thread::spawn(move || {
            loop {
                let stream = match listener.accept() {
                    Ok((stream, _)) => stream,
                    Err(error) => match error.kind() {
                        io::ErrorKind::Interrupted
                        | io::ErrorKind::ConnectionAborted
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::WouldBlock => {
                            log::debug!("Ignoring transient accept error: {}", error);
                            continue;
                        }
                        _ => {
                            log::error!("Accept failed: {}", error);
                            break;
                        }
                    },
                };

                {
                    let mut active = accept_active.lock().unwrap();
                    if *active {
                        drop(active);
                        reject_busy_client(stream);
                        continue;
                    }
                    *active = true;
                }

                if incoming_tx.send(stream).is_err() {
                    if let Ok(mut active) = accept_active.lock() {
                        *active = false;
                    }
                    break;
                }
            }
        });

        Ok(CoreListener {
            socket_path,
            client_session_active,
            incoming: incoming_rx,
            _accept_thread: accept_thread,
        })
    }

    pub fn recv(&self) -> Result<UnixStream, LkError> {
        self.incoming
            .recv()
            .map_err(|_| LkError::other("Accept thread stopped"))
    }

    pub fn session_active_flag(&self) -> Arc<Mutex<bool>> {
        self.client_session_active.clone()
    }

    pub fn release_session_claim(&self) {
        if let Ok(mut active) = self.client_session_active.lock() {
            *active = false;
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for CoreListener {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.socket_path) {
            if error.kind() != io::ErrorKind::NotFound {
                log::warn!("Failed to remove socket {}: {}", self.socket_path.display(), error);
            }
        }
    }
}

pub struct CoreServer {
    listener: CoreListener,
    runtime: CoreRuntime,
}

impl CoreServer {
    pub fn start(socket_path: PathBuf, runtime: CoreRuntime) -> Result<(), LkError> {
        let listener = CoreListener::bind(socket_path)?;
        let mut server = CoreServer { listener, runtime };
        server.run()
    }

    fn run(&mut self) -> Result<(), LkError> {
        log::info!("Listening on {}", self.listener.socket_path().display());

        loop {
            let stream = self.listener.recv()?;
            let session_active = self.listener.session_active_flag();
            if let Err(error) = run_claimed_remote_client_session(stream, &mut self.runtime, &session_active) {
                log::error!("Client session failed: {}", error);
                if error.kind == ErrorKind::Fatal {
                    return Err(error);
                }
            }
        }
    }
}
