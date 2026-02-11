/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use futures_util::io::{AsyncReadExt, AsyncWriteExt};
use hex::FromHex;
use lightkeeper_module::connection_module;

use async_ssh2_lite::{
    AsyncChannel, AsyncSession, SessionConfiguration,
    ssh2,
};
use crate::error::*;
use crate::file_handler;
use crate::file_handler::FileMetadata;
use crate::module::connection::*;
use crate::module::*;
use crate::utils::{sha256, strip_newline};

static MODULE_NAME: &str = "ssh";

type SshStream = async_ssh2_lite::AsyncIoTcpStream;


#[connection_module(
    name="ssh",
    version="0.0.1",
    description="Sends commands and file requests over SSH.",
    settings={
      port => "Port of the SSH server. Default: 22.",
      username => "Username for the SSH connection. Default: root.",
      password => "Password for the SSH connection. Stored as plaintext. Default: empty (not used).",
      private_key_path => "Path to the private key file for the SSH connection. Default: empty.",
      private_key_passphrase => "Passphrase for the private key file. Stored as plaintext. Default: empty.",
      connection_timeout => "Timeout (in seconds) for the SSH connection. Default: 15.",
      agent_key_identifier => "Identifier for selecting key from ssh-agent. This is the comment part of the \
                               key (e.g. user@desktop). Default: empty (all keys are tried).",
      verify_host_key => "Whether to verify the host key using a known_hosts-file. Default: true.",
      custom_known_hosts_path => "Path to a custom known_hosts file. Default: (inside configuration directory).",
      parallel_sessions => "Number of parallel login sessions. Improves performance. Default: 2.",
    }
)]
/// SSH connection module. Manages parallel SSH sessions internally.
pub struct Ssh2 {
    address: Arc<Mutex<String>>,
    port: Arc<Mutex<u16>>,
    username: String,
    password: Option<String>,
    private_key_path: Option<String>,
    private_key_passphrase: Option<String>,
    agent_key_identifier: Option<String>,
    connection_timeout: std::time::Duration,
    verify_host_key: bool,
    custom_known_hosts_path: Option<PathBuf>,

    session: Arc<smol::lock::Mutex<Option<AsyncSession<SshStream>>>>,
    /// Channels by invocation ID.
    channels: Arc<smol::lock::Mutex<HashMap<u64, AsyncChannel<SshStream>>>>,
}

impl Module for Ssh2 {
    fn new(settings: &HashMap<String, String>) -> Self {
        Ssh2 {
            address: Arc::new(Mutex::new(String::from("0.0.0.0"))),
            port: Arc::new(Mutex::new(settings.get("port").and_then(|value| value.parse::<u16>().ok()).unwrap_or(22))),
            username: settings.get("username").unwrap_or(&String::from("root")).clone(),
            password: settings.get("password").cloned(),
            private_key_path: settings.get("private_key_path").cloned(),
            private_key_passphrase: settings.get("private_key_passphrase").cloned(),
            agent_key_identifier: settings.get("agent_key_identifier").cloned(),
            connection_timeout: settings.get("connection_timeout")
                .and_then(|value| value.parse::<u64>().ok().map(|secs| std::time::Duration::from_secs(secs)))
                .unwrap_or(std::time::Duration::from_secs(15)),
            verify_host_key: settings.get("verify_host_key").and_then(|value| value.parse::<bool>().ok()).unwrap_or(true),
            custom_known_hosts_path: settings.get("custom_known_hosts_path").map(|path| PathBuf::from(path)),
            session: Arc::new(smol::lock::Mutex::new(None)),
            channels: Arc::new(smol::lock::Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ConnectionModule for Ssh2 {
    fn set_target(&self, address: &str) {
        let mut mutex_address = self.address.lock().unwrap();
        *mutex_address = address.to_string();
    }

    async fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        let mut response = self.send_message_binary(message, &[]).await?;
        response.message = String::from_utf8_lossy(&response.data).to_string();
        response.data = Vec::new();

        Ok(response)
    }

    async fn send_message_partial(&self, message: &str, invocation_id: u64) -> Result<ResponseMessage, LkError> {
        let session = self.get_session().await?;

        let mut channel = match session.channel_session().await {
            Ok(channel) => channel,
            Err(error) => {
                // Error is likely duo to disconnected or timeouted session. Try to reconnect once.
                log::error!("Reconnecting channel due to error: {}", error);
                self.reconnect().await
                    .map_err(|error| format!("Error reconnecting: {}", error))?;

                session.channel_session().await
                    .map_err(|error| format!("Error opening channel: {}", error))?
            }
        };

        // Merge stderr etc. to the same stream as stdout.
        channel.handle_extended_data(ssh2::ExtendedData::Merge).await?;
        channel.exec(message).await
            .map_err(|e| LkError::other(format!("Error executing command '{}': {}", message, e)))?;

        let mut buffer = [0u8; 256];
        let output = channel.read(&mut buffer).await
            .map(|bytes_read| String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
            .map_err(|e| LkError::other(format!("Invalid output: {}", e)))?;

        if channel.eof() {
            let exit_status = channel.exit_status().unwrap_or(-1);
            channel.wait_close().await
                .map_err(|e| LkError::other(format!("Error closing channel: {}", e)))?;

            Ok(ResponseMessage::new(strip_newline(&output), exit_status))
        }
        else {
            self.channels.lock().await.insert(invocation_id, channel);
            Ok(ResponseMessage::new_partial(output))
        }
    }

    async fn send_message_binary(&self, command: &str, stdin_data: &[u8]) -> Result<ResponseMessage, LkError> {
        if command.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let session = self.get_session().await?;

        let mut channel = match session.channel_session().await {
            Ok(channel) => channel,
            Err(error) => {
                log::error!("Reconnecting channel due to error: {}", error);
                self.reconnect().await?;
                session.channel_session().await
                    .map_err(|e| LkError::other(format!("Error opening channel: {}", e)))?
            }
        };

        // Merge stderr etc. to the same stream as stdout.
        channel.handle_extended_data(ssh2::ExtendedData::Merge).await?;
        channel.exec(command).await
            .map_err(|e| LkError::other(format!("Error executing command '{}': {}", command, e)))?;

        if stdin_data.len() > 0 {
            channel.write_all(stdin_data).await
                .map_err(|e| LkError::other(format!("Error writing to stdin: {}", e)))?;
            channel.send_eof().await
                .map_err(|e| LkError::other(format!("Error while closing stdin: {}", e)))?;
        }

        let mut output = Vec::new();
        channel.read_to_end(&mut output).await
            .map_err(|e| LkError::other(format!("Invalid output: {}", e)))?;

        if !channel.eof() {
            return Err(LkError::new(
                ErrorKind::Other,
                "Channel is not at EOF even though full response was requested",
            ));
        }

        let exit_status = channel.exit_status().unwrap_or(-1);
        channel.wait_close().await
            .map_err(|e| LkError::other(format!("Error closing channel: {}", e)))?;

        Ok(ResponseMessage::new_binary(output, exit_status))
    }

    async fn receive_partial_response(&self, invocation_id: u64) -> Result<ResponseMessage, LkError> {
        let mut channels = self.channels.lock().await;
        let channel = channels.get_mut(&invocation_id)
            .ok_or_else(|| LkError::other("No channel available."))?;

        let mut buffer = [0u8; 1024];
        let output = channel.read(&mut buffer).await
            .map(|bytes_read| String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
            .map_err(|e| LkError::other(format!("Invalid output: {}", e)))?;

        if channel.eof() {
            let exit_status = channel.exit_status().unwrap_or(-1);
            channel.wait_close().await
                .map_err(|e| LkError::other(format!("Error closing channel: {}", e)))?;

            channels.remove(&invocation_id);
            Ok(ResponseMessage::new(strip_newline(&output), exit_status))
        }
        else {
            Ok(ResponseMessage::new_partial(output))
        }
    }

    async fn interrupt(&self, _invocation_id: u64) -> Result<(), LkError> {
        // TODO
        // channel.process_startup("signal", Some("INT")).await.map_err(LkError::other)?;
        //let mut session_data = self.available_sessions[index].lock().unwrap();
        //session_data.open_channel = Some(channel);
        Ok(())
    }

    async fn download_file(&self, source: &str) -> Result<(FileMetadata, Vec<u8>), LkError> {
        let session = self.get_session().await?;
        let sftp = session.sftp().await?;

        let mut file = sftp.open(Path::new(source)).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let stat = file.stat().await?;

        match (stat.uid, stat.gid, stat.perm) {
            (Some(uid), Some(gid), Some(perm)) => {
                let metadata = FileMetadata {
                    download_time: Utc::now(),
                    local_path: None,
                    remote_path: source.to_string(),
                    remote_file_hash: sha256::hash(&contents),
                    owner_uid: uid,
                    owner_gid: gid,
                    permissions: perm,
                    temporary: true,
                };

                Ok((metadata, contents))
            }
            _ => {
                Err(LkError::unexpected())
            }
        }
    }

    async fn upload_file(&self, metadata: &FileMetadata, contents: Vec<u8>) -> Result<(), LkError> {
        let session = self.get_session().await?;
        let sftp = session.sftp().await?;

        let mut file = sftp.open_mode(
            Path::new(&metadata.remote_path),
            ssh2::OpenFlags::WRITE | ssh2::OpenFlags::TRUNCATE,
            metadata.permissions.try_into().unwrap(),
            ssh2::OpenType::File,
        ).await?;

        file.write_all(&contents).await?;
        Ok(())
    }

    async fn verify_host_key(&self, hostname: &str, key_id: &str) -> Result<(), LkError> {
        let session = self.get_session().await?;
        let self_port = *self.port.lock().unwrap();

        let known_hosts_path = Self::get_known_hosts_path(self.custom_known_hosts_path.as_ref())?;
        if Self::check_known_hosts(&session, hostname, self_port, Some(&known_hosts_path)).await.is_ok() {
            return Ok(());
        }
        
        let mut known_hosts = session.known_hosts()
            .map_err(|error| LkError::other(format!("Failed to initialize known hosts file: {}", error)))?;

        known_hosts.read_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
            .map_err(|error| LkError::other_p("Failed to read known hosts file", error))?;

        // The session probably gets disconnected since receiving host key fails if not reconnecting.
        // TODO: at least this was the case with ssh2-rs.
        // self.connect().await?;

        if let Some((key, key_type)) = session.host_key() {
            let key_string = Self::get_host_key_id(key_type, key)?;
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
    async fn get_session(&self) -> Result<AsyncSession<SshStream>, LkError> {
        if self.session.lock().await.is_none() {
            let session = self.connect().await?;
            let mut session_mutex = self.session.lock().await;
            *session_mutex = Some(session);
        }

        let session = self.session.lock().await;
        Ok(session.as_ref().ok_or_else(|| LkError::other("No session open"))?.clone())
    }

    async fn connect(&self) -> Result<AsyncSession<SshStream>, LkError> {
        let address = self.address.lock().unwrap().clone();
        let port = *self.port.lock().unwrap();
        let mut socket_addresses = format!("{}:{}", address, port).to_socket_addrs()?;
        let socket_address = socket_addresses.next().ok_or_else(|| LkError::other("Failed to resolve address"))?;

        let mut config = SessionConfiguration::new();
        config.set_timeout(self.connection_timeout.as_millis() as u32);

        let mut session = AsyncSession::<SshStream>::connect(socket_address, config).await?;
        log::info!("Connected to {}:{}", address, port);

        if let Err(error) = session.handshake().await {
            log::debug!("Supported Kex algs: {:?}", session.supported_algs(ssh2::MethodType::Kex).await?);
            log::debug!("Supported MacCs algs: {:?}", session.supported_algs(ssh2::MethodType::MacCs).await?);
            log::debug!("Supported HostKey algs: {:?}", session.supported_algs(ssh2::MethodType::HostKey).await?);
            log::debug!("Supported CryptCs algs: {:?}", session.supported_algs(ssh2::MethodType::CryptCs).await?);
            return Err(LkError::from(error));
        }

        if self.verify_host_key {
            Self::check_known_hosts(&session, &address, port, self.custom_known_hosts_path.as_ref()).await?;
        }

        if let Some(password) = &self.password {
            session.userauth_password(self.username.as_str(), password.as_str()).await
                .map_err(|error| LkError::other(format!("Failed to authenticate with password: {}", error)))?;
        }
        else if let Some(private_key_path) = &self.private_key_path {
            let path = Path::new(private_key_path);
            let passphrase_option = self.private_key_passphrase.as_ref().map(|pass| pass.as_str());

            session.userauth_pubkey_file(self.username.as_str(), None, path, passphrase_option).await
                .map_err(|error| LkError::other(format!("Failed to authenticate with private key: {}", error)))?;
        }
        else {
            log::debug!("Password or key is not set, using SSH agent for authentication.");
            let mut agent = session.agent()
                .map_err(|error| LkError::other(format!("Failed to connect to SSH agent: {}", error)))?;

            agent.connect().await
                .map_err(|error| LkError::other(format!("Failed to connect to SSH agent: {}", error)))?;
            agent.list_identities().await?;
            let mut valid_identities = agent.identities()?;

            if let Some(selected_id) = self.agent_key_identifier.as_ref() {
                valid_identities.retain(|identity| identity.comment() == selected_id.as_str());
            }

            for identity in valid_identities.iter() {
                log::debug!("Trying to authenticate with key \"{}\".", identity.comment());
                if agent.userauth(self.username.as_str(), identity).await.is_ok() {
                    break;
                }
            }

            if !session.authenticated() {
                return Err(LkError::other("Failed to authenticate with SSH agent."));
            }
        }

        Ok(session)
    }

    async fn reconnect(&self) -> Result<(), LkError> {
        let session = self.get_session().await?;
        session.disconnect(None, "", None).await?;
        log::debug!("Disconnected");

        self.channels.lock().await.clear();

        let session = self.connect().await?;
        let mut session_mutex = self.session.lock().await;
        *session_mutex = Some(session);

        Ok(())
    }

    async fn check_known_hosts(
        session: &AsyncSession<SshStream>,
        hostname: &str,
        port: u16,
        custom_known_hosts_path: Option<&PathBuf>)
    -> Result<(), LkError> {

        let known_hosts_path = Self::get_known_hosts_path(custom_known_hosts_path)?;

        let mut known_hosts = session.known_hosts()
            .map_err(|error| LkError::other(format!("Failed to initialize known hosts file: {}", error)))?;

        known_hosts.read_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH)
            .map_err(|error| LkError::other(format!("Failed to read known hosts file: {}", error)))?;

        if let Some((key, key_type)) = session.host_key() {
            let key_string = Self::get_host_key_id(key_type, key)?;

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

    fn get_known_hosts_path(custom_known_hosts_path: Option<&PathBuf>) -> Result<PathBuf, LkError> {
        if let Some(custom_path) = custom_known_hosts_path {
            if !custom_path.exists() {
                Err(LkError::other_p("No such file for custom_known_hosts_path", custom_path.to_string_lossy()))
            }
            else {
                Ok(custom_path.clone())
            }
        }
        else {
            let config_dir = file_handler::get_config_dir();
            let known_hosts_path = config_dir.join("known_hosts");

            // Create known_hosts if it's missing.
            if !known_hosts_path.exists() {
                log::info!("Creating file '{}'", known_hosts_path.display());
                let _ = std::fs::File::create(&known_hosts_path)?;
            }

            Ok(known_hosts_path)
        }
    }

    fn get_host_key_id(key_type: ssh2::HostKeyType, key: &[u8]) -> Result<String, LkError> {
        let fp_hex = sha256::hash(key);
        let fp_bytes = Vec::<u8>::from_hex(fp_hex.clone()).map_err(|error| LkError::other(error))?;
        let fp_base64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(fp_bytes);

        Ok(format!(
            "{:?} {}\n\nFingerprints:\nSHA256 (hex): {}\nSHA256 (base64): {}",
            key_type,
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(key),
            fp_hex,
            fp_base64
        ))
    }
}

// Simplify conversion from SSH2 errors to internal error type.
impl From<async_ssh2_lite::Error> for LkError {
    fn from(error: async_ssh2_lite::Error) -> Self {
        LkError::other(error.to_string())
    }
}
