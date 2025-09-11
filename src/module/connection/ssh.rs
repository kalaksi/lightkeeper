/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::path::PathBuf;
use std::sync::MutexGuard;
use std::time::Duration;
use std::sync::Mutex;
use std::{
    net::TcpStream,
    net::ToSocketAddrs,
    collections::HashMap,
    path::Path,
    io::Read,
    io::Write,
};

use base64::Engine;
use hex::FromHex;
use chrono::Utc;
use ssh2;
use crate::{error::*, file_handler};
use crate::file_handler::FileMetadata;
use crate::utils::{sha256, strip_newline};
use lightkeeper_module::connection_module;
use crate::module::*;
use crate::module::connection::*;

static MODULE_NAME: &str = "ssh";
const SESSION_WAIT_SLEEP: u64 = 200;


#[connection_module(
    name="ssh",
    version="0.0.1",
    description="Sends commands and file requests over SSH.",
    settings={
      port => "Port of the SSH server. Default: 22.",
      username => "Username for the SSH connection. Default: root.",
      password => "Password for the SSH connection. Default: empty (not used).",
      private_key_path => "Path to the private key file for the SSH connection. Default: empty.",
      private_key_passphrase => "Passphrase for the private key file. Default: empty.",
      connection_timeout => "Timeout (in seconds) for the SSH connection. Default: 15.",
      agent_key_identifier => "Identifier for selecting key from ssh-agent. This is the comment part of the \
                               key (e.g. user@desktop). Default: empty (all keys are tried).",
      verify_host_key => "Whether to verify the host key using a known_hosts-file. Default: true.",
      custom_known_hosts_path => "Path to a custom known_hosts file. Default: (inside configuration directory).",
      parallel_sessions => "Number of parallel login sessions. Improves performance. Default: 2.",
    }
)]
pub struct Ssh2 {
    address: Mutex<String>,
    port: Mutex<u16>,
    username: String,
    password: Option<String>,
    private_key_path: Option<String>,
    private_key_passphrase: Option<String>,
    agent_key_identifier: Option<String>,
    connection_timeout: u16,
    verify_host_key: bool,
    custom_known_hosts_path: Option<PathBuf>,

    available_sessions: Vec<Mutex<SharedSessionData>>,
}

pub struct SharedSessionData {
    is_initialized: bool,
    session: ssh2::Session,
    open_channel: Option<ssh2::Channel>,
    // For incomplete invocations, tag with the invocation ID.
    invocation_id: u64,
}

impl Module for Ssh2 {
    fn new(settings: &HashMap<String, String>) -> Self {
        let parallel_sessions = settings.get("parallel_sessions").unwrap_or(&String::from("2")).parse::<u16>().unwrap();
        let mut available_sessions = Vec::new();

        for _ in 0..parallel_sessions {
            available_sessions.push(Mutex::new(SharedSessionData {
                is_initialized: false,
                session: ssh2::Session::new().unwrap(),
                open_channel: None,
                invocation_id: 0,
            }));
        }

        Ssh2 {
            address: Mutex::new(String::from("0.0.0.0")),
            port: Mutex::new(settings.get("port").unwrap_or(&String::from("22")).parse::<u16>().unwrap()),
            username: settings.get("username").unwrap_or(&String::from("root")).clone(),
            password: settings.get("password").cloned(),
            private_key_path: settings.get("private_key_path").cloned(),
            private_key_passphrase: settings.get("private_key_passphrase").cloned(),
            agent_key_identifier: settings.get("agent_key_identifier").cloned(),
            connection_timeout: settings.get("connection_timeout").unwrap_or(&String::from("15")).parse::<u16>().unwrap(),
            verify_host_key: settings.get("verify_host_key").unwrap_or(&String::from("true")).parse::<bool>().unwrap(),
            custom_known_hosts_path: settings.get("custom_known_hosts_path").map(|path| PathBuf::from(path)),
            available_sessions: available_sessions,
        }
    }
}

impl ConnectionModule for Ssh2 {
    fn set_target(&self, address: &str) {
        let mut mutex_address = self.address.lock().unwrap();
         *mutex_address = address.to_string();
    }

    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        if message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let mut session_data = self.wait_for_session(0, true)?;

        let mut channel = match session_data.session.channel_session() {
            Ok(channel) => channel,
            Err(error) => {
                // Error is likely duo to disconnected or timeouted session. Try to reconnect once.
                log::error!("Reconnecting channel due to error: {}", error);
                self.reconnect(&mut session_data)
                    .map_err(|error| format!("Error reconnecting: {}", error))?;

                session_data.session.channel_session()
                    .map_err(|error| format!("Error opening channel: {}", error))?
            }
        };

        // Merge stderr etc. to the same stream as stdout.
        channel.handle_extended_data(ssh2::ExtendedData::Merge).unwrap();

        channel.exec(message)
               .map_err(|error| format!("Error executing command '{}': {}", message, error))?;

        let mut output = String::new();

        channel.read_to_string(&mut output)
               .map_err(|error| format!("Invalid output received: {}", error))?;

        if !channel.eof() {
            return Err(LkError::new(ErrorKind::Other, "Channel is not at EOF even though full response was requested"));
        }

        let exit_status = channel.exit_status().unwrap_or(-1);

        channel.wait_close()
               .map_err(|error| format!("Error while closing channel: {}", error))?;

        Ok(ResponseMessage::new(strip_newline(&output), exit_status))
    }

    fn send_message_partial(&self, message: &str, invocation_id: u64) -> Result<ResponseMessage, LkError> {
        let mut session_data = self.wait_for_session(0, true)?;

        let mut channel = match session_data.session.channel_session() {
            Ok(channel) => channel,
            Err(error) => {
                // Error is likely duo to disconnected or timeouted session. Try to reconnect once.
                log::error!("Reconnecting channel due to error: {}", error);
                self.reconnect(&mut session_data)
                    .map_err(|error| format!("Error reconnecting: {}", error))?;

                session_data.session.channel_session()
                    .map_err(|error| format!("Error opening channel: {}", error))?
            }
        };

        // Merge stderr etc. to the same stream as stdout.
        channel.handle_extended_data(ssh2::ExtendedData::Merge).unwrap();
        
        channel.exec(message)
               .map_err(|error| format!("Error executing command '{}': {}", message, error))?;

        let mut buffer = [0u8; 256];
        let output = channel.read(&mut buffer)
            .map(|bytes_read| String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
            .map_err(|error| format!("Invalid output received: {}", error))?;

        if channel.eof() {
            let exit_status = channel.exit_status().unwrap_or(-1);
            channel.wait_close()
                   .map_err(|error| format!("Error while closing channel: {}", error))?;

            Ok(ResponseMessage::new(strip_newline(&output), exit_status))
        }
        else {
            session_data.invocation_id = invocation_id;
            session_data.open_channel = Some(channel);
            Ok(ResponseMessage::new_partial(output))
        }
    }

    fn receive_partial_response(&self, invocation_id: u64) -> Result<ResponseMessage, LkError> {
        let mut partial_session = self.wait_for_session(invocation_id, true)?;
        let mut channel = match partial_session.open_channel.take() {
            Some(channel) => channel,
            None => return Err(LkError::other("Can't do partial receive. No open channel available.")),
        };

        let mut buffer = [0u8; 1024];
        let output = channel.read(&mut buffer)
            .map(|bytes_read| String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
            .map_err(|error| {
                partial_session.invocation_id = 0;
                format!("Invalid output received: {}", error)
            })?;

        if channel.eof() {
            partial_session.invocation_id = 0;
            let exit_status = channel.exit_status().unwrap_or(-1);

            channel.wait_close()
                .map_err(|error| format!("Error while closing channel: {}", error))?;

            Ok(ResponseMessage::new(strip_newline(&output), exit_status))
        }
        else {
            partial_session.open_channel = Some(channel);
            Ok(ResponseMessage::new_partial(output))
        }
    }

    fn download_file(&self, source: &str) -> Result<(FileMetadata, Vec<u8>), LkError> {
        let session_data = self.wait_for_session(0, true)?;
        let sftp = session_data.session.sftp()?;

        let mut file = sftp.open(Path::new(&source))?;
        let mut contents = Vec::new();
        let _bytes_written = file.read_to_end(&mut contents)?;
        let stat = file.stat()?;
        let metadata = FileMetadata {
            download_time: Utc::now(),
            local_path: None,
            remote_path: source.to_string(),
            remote_file_hash: sha256::hash(&contents),
            owner_uid: stat.uid.unwrap(),
            owner_gid: stat.gid.unwrap(),
            permissions: stat.perm.unwrap(),
            temporary: true,
        };

        Ok((metadata, contents))
    }

    fn upload_file(&self, metadata: &FileMetadata, contents: Vec<u8>) -> Result<(), LkError> {
        let session_data = self.wait_for_session(0, true)?;
        let sftp = session_data.session.sftp()?;

        let file = sftp.open_mode(
            Path::new(&metadata.remote_path),
            ssh2::OpenFlags::WRITE | ssh2::OpenFlags::TRUNCATE,
            metadata.permissions.try_into().unwrap(),
            ssh2::OpenType::File,
        );

        file?.write(&contents)
             .map(|_| Ok(()))?
    }

    fn verify_host_key(&self, hostname: &str, key_id: &str) -> Result<(), LkError> {
        let mut session_data = self.wait_for_session(0, false)?;
        let self_address = self.address.lock().unwrap().to_string();
        let self_port = *self.port.lock().unwrap();

        // One last check to avoid writing duplicates. Can otherwise happen with parallel SSH sessions.
        if self.check_known_hosts(&session_data, hostname, self_port).is_ok() {
            return Ok(());
        }

        let known_hosts_path = self.get_known_hosts_path()?;

        let mut known_hosts = session_data.session.known_hosts().unwrap();
        known_hosts.read_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
                   .map_err(|error| LkError::other_p("Failed to read known hosts file", error))?;

        // The session probably gets disconnected since receiving host key fails if not reconnecting.
        let mut socket_addresses = format!("{}:{}", self_address, self_port).to_socket_addrs()?;
        let socket_address = match socket_addresses.next() {
            Some(address) => address,
            None => return Err(LkError::other("Failed to resolve address")),
        };

        let connection_timeout = std::time::Duration::from_secs(self.connection_timeout as u64);
        let stream = TcpStream::connect_timeout(&socket_address, connection_timeout)?;

        log::info!("Connected to {}:{}", self_address, self_port);
        session_data.session = ssh2::Session::new().unwrap();
        session_data.session.set_tcp_stream(stream);
        session_data.session.handshake()?;

        if let Some((key, key_type)) = session_data.session.host_key() {
            let key_string = Self::get_host_key_id(key_type, key);
            let host_and_port = format!("[{}]:{}", hostname, self_port);

            if key_string == key_id {
                known_hosts.add(&host_and_port, key, hostname, key_type.into())
                           .map_err(|error| LkError::other_p("Failed to add host key to known hosts", error))?;
                known_hosts.write_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
                           .map_err(|error| LkError::other_p("Failed to write known hosts file", error))?;
                Ok(())
            }
            else {
                Err(LkError::other("Host key changed again?!"))
            }
        }
        else {
            Err(LkError::other("Failed to get host key"))
        }
    }
}

impl Ssh2 {
    fn wait_for_session(&self, invocation_id: u64, connect_automatically: bool) -> Result<MutexGuard<SharedSessionData>, LkError> {
        let mut total_wait = Duration::from_secs(0);

        loop {
            for (index, session) in self.available_sessions.iter().enumerate() {
                if let Ok(mut session_data) = session.try_lock() {
                    // Incomplete commands will want a specific invocation. ID 0 means not used.
                    if session_data.invocation_id > 0 && session_data.invocation_id != invocation_id {
                        continue;
                    }

                    log::debug!("Attached to session {}", index);

                    if connect_automatically && !session_data.is_initialized {
                        let address = self.address.lock().unwrap().clone();
                        let port = *self.port.lock().unwrap();
                        if let Err(error) = self.connect(&mut session_data, &address, port) {
                            log::error!("Error while connecting {}: {}", address, error);
                            return Err(error);
                        }
                    }

                    return Ok(session_data);
                }
            }

            std::thread::sleep(Duration::from_millis(SESSION_WAIT_SLEEP));
            total_wait += Duration::from_millis(SESSION_WAIT_SLEEP);

            // Print a warning every 2 minutes.
            if total_wait.as_secs() % 120 == 0 {
                log::warn!("No free SSH session available after {} seconds. Still waiting.", total_wait.as_secs());
            }
        }
    }

    fn connect(&self, session_data: &mut MutexGuard<SharedSessionData>, address: &str, port: u16) -> Result<(), LkError> {
        if session_data.is_initialized {
            return Ok(())
        }

        let mut socket_addresses = format!("{}:{}", address, port).to_socket_addrs()?;
        let socket_address = match socket_addresses.next() {
            Some(address) => address,
            None => return Err(LkError::other("Failed to resolve address")),
        };

        let connection_timeout = std::time::Duration::from_secs(self.connection_timeout as u64);
        let stream = TcpStream::connect_timeout(&socket_address, connection_timeout)?;
        log::info!("Connected to {}:{}", address, port);

        session_data.session = ssh2::Session::new().unwrap();
        session_data.session.set_tcp_stream(stream);
        if let Err(error) = session_data.session.handshake() {
            log::debug!("Supported Kex algs: {:?}", session_data.session.supported_algs(ssh2::MethodType::Kex));
            log::debug!("Supported MacCs algs: {:?}", session_data.session.supported_algs(ssh2::MethodType::MacCs));
            log::debug!("Supported HostKey algs: {:?}", session_data.session.supported_algs(ssh2::MethodType::HostKey));
            log::debug!("Supported CryptCs algs: {:?}", session_data.session.supported_algs(ssh2::MethodType::CryptCs));
            return Err(LkError::from(error))
        }

        if self.verify_host_key {
            self.check_known_hosts(&session_data, &address, port)?;
        }

        if self.password.is_some() {
            session_data.session.userauth_password(self.username.as_str(), self.password.as_ref().unwrap().as_str())
                .map_err(|error| LkError::other(format!("Failed to authenticate with password: {}", error)))?;
        }
        else if self.private_key_path.is_some() {
            let path = Path::new(self.private_key_path.as_ref().unwrap());
            let passphrase_option = self.private_key_passphrase.as_ref().map(|pass| pass.as_str());

            session_data.session.userauth_pubkey_file(self.username.as_str(), None, path, passphrase_option)
                .map_err(|error| LkError::other(format!("Failed to authenticate with private key: {}", error)))?;
        }
        else {
            log::debug!("Password or key is not set, using SSH agent for authentication.");
            let mut agent = session_data.session.agent()
                .map_err(|error| LkError::other(format!("Failed to connect to SSH agent: {}", error)))?;

            agent.connect()
                 .map_err(|error| LkError::other(format!("Failed to connect to SSH agent: {}", error)))?;

            agent.list_identities()?;
            let mut valid_identities = agent.identities()?;

            if let Some(selected_id) = self.agent_key_identifier.as_ref() {
                valid_identities.retain(|identity| identity.comment() == selected_id.as_str());
            }

            for identity in valid_identities.iter() {
                log::debug!("Trying to authenticate with key \"{}\".", identity.comment());
                if agent.userauth(self.username.as_str(), identity).is_ok() {
                    break;
                }
            }

            if !session_data.session.authenticated() {
                return Err(LkError::other("Failed to authenticate with SSH agent."));
            }
        }

        session_data.is_initialized = true;
        Ok(())
    }

    fn reconnect(&self, session_data: &mut MutexGuard<SharedSessionData>) -> Result<(), LkError> {
        let address = self.address.lock().unwrap().clone();
        let port = *self.port.lock().unwrap();

        session_data.session.disconnect(None, "", None)?;
        session_data.is_initialized = false;
        log::debug!("Disconnected");
        self.connect(session_data, &address, port)
    }

    fn check_known_hosts(&self, session_data: &MutexGuard<SharedSessionData>, hostname: &str, port: u16) -> Result<(), LkError> {
        let known_hosts_path = self.get_known_hosts_path()?;

        let mut known_hosts = session_data.session.known_hosts().unwrap();
        known_hosts.read_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
                   .map_err(|error| LkError::other(format!("Failed to read known hosts file: {}", error)))?;

        if let Some((key, key_type)) = session_data.session.host_key() {
            let key_string = Self::get_host_key_id(key_type, key);

            match known_hosts.check_port(hostname, port, key) {
                ssh2::CheckResult::Match => Ok(()),
                ssh2::CheckResult::NotFound => {
                    let message = format!("Host key for '{}' was not found in known hosts.\nDo you want to trust this key:", hostname);
                    Err(LkError::host_key_unverified(MODULE_NAME, &message, &key_string))
                },
                ssh2::CheckResult::Mismatch => {
                    let message = format!("Host key for '{}' HAS CHANGED! Do you trust this NEW key:", hostname);
                    Err(LkError::host_key_unverified(MODULE_NAME, &message, &key_string))
                },
                ssh2::CheckResult::Failure => {
                    let message = format!("Failed to get host key for '{}'", hostname);
                    Err(LkError::other(message))
                }
            }
        }
        else {
            Err(LkError::other("Failed to get host key"))
        }
    }

    fn get_known_hosts_path(&self) -> Result<PathBuf, LkError> {
        if self.custom_known_hosts_path.is_some() {
            let known_hosts_path = self.custom_known_hosts_path.as_ref().unwrap();

            if !known_hosts_path.exists() {
                return Err(LkError::other_p("No such file for custom_known_hosts_path", known_hosts_path.to_string_lossy()));
            }

            Ok(known_hosts_path.clone())
        }
        else {
            let config_dir = file_handler::get_config_dir()?;
            let known_hosts_path = config_dir.join("known_hosts");

            // Create known_hosts if it's missing.
            if !known_hosts_path.exists() {
                log::info!("Creating file '{}'", known_hosts_path.display());
                let _ = std::fs::File::create(&known_hosts_path)?;
            }

            Ok(known_hosts_path)
        }
    }

    fn get_host_key_id(key_type: ssh2::HostKeyType, key: &[u8]) -> String {
        let fp_hex = sha256::hash(key);
        let fp_bytes = Vec::<u8>::from_hex(fp_hex.clone()).unwrap();
        let fp_base64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(fp_bytes);

        format!("{:?} {}\n\nFingerprints:\nSHA256 (hex): {}\nSHA256 (base64): {}",
            key_type,
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(key),
            fp_hex,
            fp_base64
        )
    }
}


/// Simplify conversion from SSH2 errors to internal error type.
/// See: https://github.com/alexcrichton/ssh2-rs/blob/master/libssh2-sys/lib.rs
impl From<ssh2::Error> for LkError {
    fn from(error: ssh2::Error) -> Self {
        match error.code() {
            ssh2::ErrorCode::Session(-5) => LkError::new(ErrorKind::ConnectionFailed, error),
            _ => LkError::new(ErrorKind::Other, error),
        }
    }
}