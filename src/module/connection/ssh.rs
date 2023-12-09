use std::{
    net::TcpStream,
    net::IpAddr,
    net::Ipv4Addr,
    net::ToSocketAddrs,
    collections::HashMap,
    path::Path,
    io,
    io::Read,
    io::Write,
};

use chrono::Utc;
use ssh2::Session;
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
                               key (e.g. user@desktop). Default: empty (all keys are tried)."
    }
)]
pub struct Ssh2 {
    session: Session,
    is_initialized: bool,
    address: IpAddr,
    port: u16,
    username: String,
    password: Option<String>,
    private_key_path: Option<String>,
    private_key_passphrase: Option<String>,
    agent_key_identifier: Option<String>,
    connection_timeout: u16,
}

impl Module for Ssh2 {
    fn new(settings: &HashMap<String, String>) -> Self {
        // This can fail for unknown reasons, no way to propery handle the error.
        let session = Session::new().unwrap();

        Ssh2 {
            session: session,
            is_initialized: false,
            address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: settings.get("port").unwrap_or(&String::from("22")).parse::<u16>().unwrap(),
            username: settings.get("username").unwrap_or(&String::from("root")).clone(),
            password: settings.get("password").cloned(),
            private_key_path: settings.get("private_key_path").cloned(),
            // TODO: Hide passphrase in UI (currently behaves like a normal text field).
            private_key_passphrase: settings.get("private_key_passphrase").cloned(),
            agent_key_identifier: settings.get("agent_key_identifier").cloned(),
            connection_timeout: settings.get("connection_timeout").unwrap_or(&String::from("15")).parse::<u16>().unwrap(),
        }
    }
}

impl ConnectionModule for Ssh2 {
    fn connect(&mut self, address: &IpAddr) -> Result<(), String> {
        if self.is_initialized {
            return Ok(())
        }

        self.address = address.clone();

        let socket_address = format!("{}:{}", address, self.port).to_socket_addrs().unwrap().next().unwrap();
        let connection_timeout = std::time::Duration::from_secs(self.connection_timeout as u64);
        let stream = match TcpStream::connect_timeout(&socket_address, connection_timeout) {
            Ok(stream) => stream,
            Err(error) => return Err(error.to_string())
        };

        log::info!("Connected to {}:{}", address, self.port);

        self.session = Session::new().unwrap();
        self.session.set_tcp_stream(stream);
        if let Err(error) = self.session.handshake() {
            return Err(format!("Handshake error: {}", error));
        };

        if self.password.is_some() {
            if let Err(error) = self.session.userauth_password(self.username.as_str(), self.password.as_ref().unwrap().as_str()) {
                return Err(format!("Failed to authenticate with password: {}", error));
            };
        }
        else if self.private_key_path.is_some() {
            let path = Path::new(self.private_key_path.as_ref().unwrap());
            let passphrase_option = self.private_key_passphrase.as_ref().map(|pass| pass.as_str());

            if let Err(error) = self.session.userauth_pubkey_file(self.username.as_str(), None, path, passphrase_option) {
                return Err(format!("Failed to authenticate with private key: {}", error));
            };
        }
        else {
            log::debug!("Password or key is not set, using SSH agent for authentication.");
            let mut agent = self.session.agent()
                .map_err(|error| format!("Failed to connect to SSH agent: {}", error))?;

            agent.connect()
                .map_err(|error| format!("Failed to connect to SSH agent: {}", error))?;

            agent.list_identities().map_err(|error| error.to_string())?;
            let mut valid_identities = agent.identities().map_err(|error| error.to_string())?;

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
                return Err(format!("Failed to authenticate with SSH agent."));
            }
        }

        self.is_initialized = true;
        Ok(())
    }

    fn send_message(&mut self, message: &str) -> Result<ResponseMessage, String> {
        if message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let mut channel = match self.session.channel_session() {
            Ok(channel) => channel,
            Err(error) => {
                // Error is likely duo to disconnected or timeouted session. Try to reconnect once.
                log::error!("Reconnecting channel due to error: {}", error);
                if let Err(error) = self.reconnect() {
                    return Err(format!("Error reconnecting: {}", error));
                }

                match self.session.channel_session() {
                    Ok(channel) => channel,
                    Err(error) => return Err(format!("Error opening channel: {}", error))
                }
            }
        };

        if let Err(error) = channel.exec(message) {
            return Err(format!("Error executing command '{}': {}", message, error));
        };

        let mut output = String::new();
        if let Err(error) = channel.read_to_string(&mut output) {
            return Err(format!("Invalid output string received: {}", error));
        };

        let exit_status = channel.exit_status().unwrap_or(-1);

        if let Err(error) = channel.wait_close() {
            log::error!("Error while closing channel: {}", error);
        };

        Ok(ResponseMessage::new(strip_newline(&output), exit_status))
    }

    fn download_file(&self, source: &String) -> io::Result<(FileMetadata, Vec<u8>)> {
        let sftp = self.session.sftp().unwrap();
        match sftp.open(Path::new(&source)) {
            Ok(mut file) => {
                let mut contents = Vec::new();
                if let Err(error) = file.read_to_end(&mut contents) {
                    Err(error)
                }
                else {
                    let stat = file.stat().unwrap();
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
            }
            Err(error) => {
                Err(io::Error::new(io::ErrorKind::Other, error.message()))
            }
        }
    }

    fn upload_file(&self, metadata: &FileMetadata, contents: Vec<u8>) -> io::Result<()> {
        let sftp = self.session.sftp().unwrap();

        let file = sftp.open_mode(
            Path::new(&metadata.remote_path),
            ssh2::OpenFlags::WRITE | ssh2::OpenFlags::TRUNCATE,
            metadata.permissions.try_into().unwrap(),
            ssh2::OpenType::File,
        );

        match file {
            Ok(mut file) => {
                file.write(&contents)?;
                Ok(())
            }
            Err(error) => {
                Err(io::Error::new(io::ErrorKind::Other, error.message()))
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.is_initialized
    }

    fn reconnect(&mut self) -> Result<(), String> {
        self.disconnect();
        log::debug!("Disconnected");
        self.connect(&self.address.clone())
    }

    fn disconnect(&mut self) {
        let _ = self.session.disconnect(None, "", None);
        self.is_initialized = false;
    }
}

