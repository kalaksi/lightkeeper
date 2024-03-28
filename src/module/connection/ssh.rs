use std::{
    net::TcpStream,
    net::ToSocketAddrs,
    collections::HashMap,
    path::Path,
    io::Read,
    io::Write,
};

use chrono::Utc;
use ssh2;
use crate::error::*;
use crate::file_handler::FileMetadata;
use crate::utils::strip_newline;
use lightkeeper_module::connection_module;
use crate::module::*;
use crate::module::connection::*;

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
    }
)]
pub struct Ssh2 {
    session: ssh2::Session,
    is_initialized: bool,
    address: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key_path: Option<String>,
    private_key_passphrase: Option<String>,
    agent_key_identifier: Option<String>,
    connection_timeout: u16,
    verify_host_key: bool,
    // TODO: multiple parallel channels for multiple commands?
    open_channel: Option<ssh2::Channel>,
}

impl Module for Ssh2 {
    fn new(settings: &HashMap<String, String>) -> Self {
        // This can fail for unknown reasons, no way to propery handle the error.
        let session = ssh2::Session::new().unwrap();

        Ssh2 {
            session: session,
            is_initialized: false,
            address: String::from("0.0.0.0"),
            port: settings.get("port").unwrap_or(&String::from("22")).parse::<u16>().unwrap(),
            username: settings.get("username").unwrap_or(&String::from("root")).clone(),
            password: settings.get("password").cloned(),
            private_key_path: settings.get("private_key_path").cloned(),
            // TODO: Hide passphrase in UI (currently behaves like a normal text field).
            private_key_passphrase: settings.get("private_key_passphrase").cloned(),
            agent_key_identifier: settings.get("agent_key_identifier").cloned(),
            connection_timeout: settings.get("connection_timeout").unwrap_or(&String::from("15")).parse::<u16>().unwrap(),
            verify_host_key: settings.get("verify_host_key").unwrap_or(&String::from("true")).parse::<bool>().unwrap(),
            open_channel: None,
        }
    }
}

impl ConnectionModule for Ssh2 {
    fn connect(&mut self, address: &str) -> Result<(), LkError> {
        if self.is_initialized {
            return Ok(())
        }

        self.address = address.to_string();
        let socket_address = format!("{}:{}", self.address, self.port).to_socket_addrs().unwrap().next().unwrap();
        let connection_timeout = std::time::Duration::from_secs(self.connection_timeout as u64);
        let stream = TcpStream::connect_timeout(&socket_address, connection_timeout)?;

        log::info!("Connected to {}:{}", address, self.port);

        self.session = ssh2::Session::new().unwrap();
        self.session.set_tcp_stream(stream);
        self.session.handshake()?;

        if self.verify_host_key {
            self.verify_host_key(address)?;
        }

        if self.password.is_some() {
            self.session.userauth_password(self.username.as_str(), self.password.as_ref().unwrap().as_str())
                .map_err(|error| LkError::new_other(format!("Failed to authenticate with password: {}", error)))?;
        }
        else if self.private_key_path.is_some() {
            let path = Path::new(self.private_key_path.as_ref().unwrap());
            let passphrase_option = self.private_key_passphrase.as_ref().map(|pass| pass.as_str());

            self.session.userauth_pubkey_file(self.username.as_str(), None, path, passphrase_option)
                .map_err(|error| LkError::new_other(format!("Failed to authenticate with private key: {}", error)))?;
        }
        else {
            log::debug!("Password or key is not set, using SSH agent for authentication.");
            let mut agent = self.session.agent()
                .map_err(|error| LkError::new_other(format!("Failed to connect to SSH agent: {}", error)))?;

            agent.connect()
                 .map_err(|error| LkError::new_other(format!("Failed to connect to SSH agent: {}", error)))?;

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

            if !self.session.authenticated() {
                return Err(LkError::new_other("Failed to authenticate with SSH agent."));
            }
        }

        self.is_initialized = true;
        Ok(())
    }

    fn send_message(&mut self, message: &str, wait_full_result: bool) -> Result<ResponseMessage, LkError> {
        if message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let mut channel = match self.session.channel_session() {
            Ok(channel) => channel,
            Err(error) => {
                // Error is likely duo to disconnected or timeouted session. Try to reconnect once.
                log::error!("Reconnecting channel due to error: {}", error);
                self.reconnect()
                    .map_err(|error| format!("Error reconnecting: {}", error))?;

                self.session.channel_session()
                    .map_err(|error| format!("Error opening channel: {}", error))?
            }
        };

        // Merge stderr etc. to the same stream as stdout.
        channel.handle_extended_data(ssh2::ExtendedData::Merge).unwrap();

        channel.exec(message)
               .map_err(|error| format!("Error executing command '{}': {}", message, error))?;

        let mut output = String::new();

        if wait_full_result {
            channel.read_to_string(&mut output)
                   .map_err(|error| format!("Invalid output received: {}", error))?;
        }
        else {
            let mut buffer = [0u8; 256];
            output = channel.read(&mut buffer)
                .map(|_| std::str::from_utf8(&buffer).unwrap().to_string())
                .map_err(|error| format!("Invalid output received: {}", error))?;
        }

        if channel.eof() {
            let exit_status = channel.exit_status().unwrap_or(-1);

            channel.wait_close()
                   .map_err(|error| format!("Error while closing channel: {}", error))?;

            Ok(ResponseMessage::new(strip_newline(&output), exit_status))
        }
        else {
            if wait_full_result {
                panic!("Channel is not at EOF even though full response was requested");
            }

            self.open_channel = Some(channel);
            Ok(ResponseMessage::new_partial(output))
        }
    }

    fn receive_partial_response(&mut self) -> Result<ResponseMessage, LkError> {
        if let Some(channel) = &mut self.open_channel {
            let mut buffer = [0u8; 1024];
            let output = channel.read(&mut buffer)
                .map(|count| std::str::from_utf8(&buffer[0..count]).unwrap().to_string())
                .map_err(|error| format!("Invalid output received: {}", error))?;

            if channel.eof() {
                let exit_status = channel.exit_status().unwrap_or(-1);

                channel.wait_close()
                       .map_err(|error| format!("Error while closing channel: {}", error))?;

                self.open_channel = None;
                Ok(ResponseMessage::new(strip_newline(&output), exit_status))
            }
            else {
                Ok(ResponseMessage::new_partial(output))
            }
        }
        else {
            panic!("No open channel available");
        }
    }

    fn download_file(&self, source: &String) -> Result<(FileMetadata, Vec<u8>), LkError> {
        let sftp = self.session.sftp()?;

        let mut file = sftp.open(Path::new(&source))?;
        let mut contents = Vec::new();
        let _bytes_written = file.read_to_end(&mut contents)?;
        let stat = file.stat()?;
        let metadata = FileMetadata {
            download_time: Utc::now(),
            local_path: None,
            remote_path: source.clone(),
            remote_file_hash: sha256::digest(contents.as_slice()),
            owner_uid: stat.uid.unwrap(),
            owner_gid: stat.gid.unwrap(),
            permissions: stat.perm.unwrap(),
            temporary: true,
        };

        Ok((metadata, contents))
    }

    fn upload_file(&self, metadata: &FileMetadata, contents: Vec<u8>) -> Result<(), LkError> {
        let sftp = self.session.sftp()?;

        let file = sftp.open_mode(
            Path::new(&metadata.remote_path),
            ssh2::OpenFlags::WRITE | ssh2::OpenFlags::TRUNCATE,
            metadata.permissions.try_into().unwrap(),
            ssh2::OpenType::File,
        );

        file?.write(&contents)
             .map(|_| Ok(()))?
    }

    fn is_connected(&self) -> bool {
        self.is_initialized
    }

    fn reconnect(&mut self) -> Result<(), LkError> {
        self.disconnect();
        log::debug!("Disconnected");
        self.connect(&self.address.clone())
    }

    fn disconnect(&mut self) {
        let _ = self.session.disconnect(None, "", None);
        self.is_initialized = false;
    }
}

impl Ssh2 {
    fn verify_host_key(&self, hostname: &str) -> Result<(), LkError> {
        let known_hosts_path = Path::new(&std::env::var("HOME").unwrap()).join(".ssh/known_hosts");
        let mut known_hosts = self.session.known_hosts().unwrap();
        if let Err(error) = known_hosts.read_file(&known_hosts_path, ssh2::KnownHostFileKind::OpenSSH) {
            log::warn!("Failed to read known hosts file: {}", error);
        }

        if let Some((key, _key_type)) = self.session.host_key() {
            match known_hosts.check(hostname, key) {
                ssh2::CheckResult::Match => Ok(()),
                ssh2::CheckResult::NotFound => {
                    let message = format!("Host key for '{}' not found in known hosts", hostname);
                    Err(LkError::new(ErrorKind::HostKeyNotVerified, message))
                    // known_hosts.add(host, key, host, key_type.into()).unwrap();
                    // known_hosts.write_file(&file, KnownHostFileKind::OpenSSH).unwrap();
                },
                ssh2::CheckResult::Mismatch => {
                    let message = format!("Host key for '{}' does not match the known hosts", hostname);
                    Err(LkError::new(ErrorKind::HostKeyNotVerified, message))
                },
                ssh2::CheckResult::Failure => {
                    let message = format!("Failed to get host key for '{}'", hostname);
                    Err(LkError::new(ErrorKind::HostKeyNotVerified, message))
                }
            }
        }
        else {
            Err(LkError::new(ErrorKind::HostKeyNotVerified, "Failed to get host key"))
        }
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