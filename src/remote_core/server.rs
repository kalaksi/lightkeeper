/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::configuration::{Configuration, Groups, Hosts};
use crate::error::LkError;
use crate::remote_core::protocol::{
    read_message, ClientMessage, ServerMessage, PROTOCOL_VERSION,
};
use crate::remote_core::runtime::CoreRuntime;
use crate::remote_core::session::RemoteSession;

pub fn run_remote_client_session(
    mut stream: UnixStream,
    runtime: &mut CoreRuntime,
    client_session_active: &Arc<Mutex<bool>>,
) -> Result<(), LkError> {
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
        session.send_error(format!(
            "Unsupported protocol version {}. Expected {}.",
            protocol_version, PROTOCOL_VERSION,
        ))?;
        return Ok(());
    }

    {
        let mut active = client_session_active.lock().unwrap();
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
        flag: client_session_active.clone(),
    };

    handle_connected_client_loop(&mut stream, runtime)
}

fn handle_connected_client_loop(stream: &mut UnixStream, runtime: &mut CoreRuntime) -> Result<(), LkError> {
    let update_receiver = runtime.new_update_receiver();
    let mut session = RemoteSession::new(stream.try_clone()?);
    session.send_message(&ServerMessage::Connect {
        protocol_version: PROTOCOL_VERSION,
    })?;
    session.send_message(&ServerMessage::InitialState(runtime.display_data()))?;
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
                session.send_error("Connect may only be sent once")?;
            }
            ClientMessage::ExecuteCommand {
                request_id,
                host_id,
                command_id,
                parameters,
            } => {
                match runtime.execute_command(&host_id, &command_id, &parameters) {
                    Ok(invocation_id) => {
                        session.send_message(&ServerMessage::ExecuteCommand {
                            request_id,
                            invocation_id,
                        })?;
                    }
                    Err(error) => {
                        session.send_message(&ServerMessage::Error {
                            request_id: Some(request_id),
                            message: error.to_string(),
                        })?;
                    }
                }
            }
            ClientMessage::CommandsForHost { request_id, host_id } => {
                session.send_message(&ServerMessage::CommandsForHost {
                    request_id,
                    host_id: host_id.clone(),
                    commands: runtime.commands_for_host(&host_id),
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
                    command: runtime.command_for_host(&host_id, &command_id),
                })?;
            }
            ClientMessage::CustomCommandsForHost { request_id, host_id } => {
                session.send_message(&ServerMessage::CustomCommandsForHost {
                    request_id,
                    host_id: host_id.clone(),
                    commands: runtime.custom_commands_for_host(&host_id),
                })?;
            }
            ClientMessage::AllHostCategories { request_id, host_id } => {
                session.send_message(&ServerMessage::AllHostCategories {
                    request_id,
                    host_id: host_id.clone(),
                    categories: runtime.all_host_categories(&host_id),
                })?;
            }
            ClientMessage::VerifyHostKey { host_id, connector_id, key_id } => {
                runtime.verify_host_key(&host_id, &connector_id, &key_id);
            }
            ClientMessage::InterruptInvocation { invocation_id } => {
                runtime.interrupt_invocation(invocation_id);
            }
            ClientMessage::RefreshHostMonitors { host_id } => {
                runtime.refresh_host_monitors(&host_id);
            }
            ClientMessage::RefreshPlatformInfo { host_id } => {
                runtime.refresh_platform_info(&host_id);
            }
            ClientMessage::RefreshPlatformInfoAll { request_id } => {
                let host_ids = runtime.refresh_platform_info_all();
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
                let invocation_ids = runtime.refresh_monitors_for_command(&host_id, &command_id);
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
                let invocation_ids = runtime.refresh_monitors_of_category(&host_id, &category);
                session.send_message(&ServerMessage::RefreshInvocationIds {
                    request_id,
                    invocation_ids,
                })?;
            }
            ClientMessage::RefreshCertificateMonitors { request_id } => {
                let invocation_ids = runtime.refresh_certificate_monitors();
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
                let path = runtime.resolve_text_editor_path(&host_id, &command_id, &parameters);
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
                match runtime.download_editable_file(&host_id, &command_id, &remote_file_path) {
                    Ok(invocation_id) => {
                        session.send_message(&ServerMessage::DownloadEditableFileResult {
                            request_id,
                            invocation_id,
                        })?;
                    }
                    Err(error) => {
                        session.send_message(&ServerMessage::Error {
                            request_id: Some(request_id),
                            message: error.to_string(),
                        })?;
                    }
                }
            }
            ClientMessage::WriteCachedFile {
                request_id,
                host_id,
                remote_file_path,
                contents,
            } => {
                let path = runtime.core.command_handler.cache_file_path_for_remote(&host_id, &remote_file_path);
                runtime.core.command_handler.write_file(&path, contents);
                session.send_message(&ServerMessage::WriteCachedFileResult { request_id })?;
            }
            ClientMessage::RemoveCachedFile {
                request_id,
                host_id,
                remote_file_path,
            } => {
                let path = runtime.core.command_handler.cache_file_path_for_remote(&host_id, &remote_file_path);
                runtime.core.command_handler.remove_file(&path);
                session.send_message(&ServerMessage::RemoveCachedFileResult { request_id })?;
            }
            ClientMessage::HasCachedFileChanged {
                request_id,
                host_id,
                remote_file_path,
                content_hash,
            } => {
                let changed = match runtime.core.command_handler.has_file_changed(
                    &host_id,
                    &remote_file_path,
                    &content_hash,
                ) {
                    Ok(changed) => changed,
                    Err(error) => {
                        log::error!("{}", error);
                        false
                    },
                };
                session.send_message(&ServerMessage::HasCachedFileChangedResult {
                    request_id,
                    changed,
                })?;
            }
            ClientMessage::UploadFileFromCache {
                request_id,
                host_id,
                command_id,
                remote_file_path,
            } => {
                let path = runtime.core.command_handler.cache_file_path_for_remote(&host_id, &remote_file_path);
                match runtime
                    .core
                    .command_handler
                    .upload_file(&host_id, &command_id, &path)
                {
                    Ok(invocation_id) => {
                        session.send_message(&ServerMessage::UploadFileFromCacheResult {
                            request_id,
                            invocation_id,
                        })?;
                    }
                    Err(error) => {
                        session.send_message(&ServerMessage::Error {
                            request_id: Some(request_id),
                            message: error.to_string(),
                        })?;
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
                        session.send_message(&ServerMessage::Error {
                            request_id: Some(request_id),
                            message: error.to_string(),
                        })?;
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
                let parsed: Result<(Configuration, Hosts, Groups), LkError> = (|| {
                    let main: Configuration = serde_yaml::from_str(&main_yml)?;
                    let hosts: Hosts = serde_yaml::from_str(&hosts_yml)?;
                    let groups: Groups = serde_yaml::from_str(&groups_yml)?;
                    Ok((main, hosts, groups))
                })();
                match parsed {
                    Ok((main, hosts, groups)) => {
                        let update_result: Result<(), LkError> = (|| {
                            Configuration::write_main_config(&runtime.config_dir, &main)?;
                            Configuration::write_hosts_config(&runtime.config_dir, &hosts)?;
                            Configuration::write_groups_config(&runtime.config_dir, &groups)?;
                            let module_factory = runtime.core.module_factory.clone();
                            runtime.stop();
                            let (main_read, hosts_read, _groups) = Configuration::read(&runtime.config_dir)?;
                            runtime.core = crate::initialize_core_with_module_factory(
                                &main_read,
                                &hosts_read,
                                module_factory,
                            )?;
                            Ok(())
                        })();
                        match update_result {
                            Ok(()) => {
                                session.send_message(&ServerMessage::InitialState(runtime.display_data()))?;
                                session.start_update_stream(runtime.new_update_receiver());
                                session.send_message(&ServerMessage::UpdateConfigOk { request_id })?;
                            }
                            Err(error) => {
                                session.start_update_stream(runtime.new_update_receiver());
                                session.send_message(&ServerMessage::Error {
                                    request_id: Some(request_id),
                                    message: error.to_string(),
                                })?;
                            }
                        }
                    },
                    Err(error) => {
                        session.start_update_stream(runtime.new_update_receiver());
                        session.send_message(&ServerMessage::Error {
                            request_id: Some(request_id),
                            message: error.to_string(),
                        })?;
                    }
                }
            }
        }
    }
}

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

    fn handle_client(&mut self, stream: UnixStream) -> Result<(), LkError> {
        run_remote_client_session(stream, &mut self.runtime, &self.client_session_active)
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
