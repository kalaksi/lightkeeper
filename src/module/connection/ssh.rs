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
      connection_timeout => "Timeout (in seconds) for the SSH connection. Default: 15."
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
    connection_timeout: u16,
}

impl Ssh2 {
    fn send_eof_and_close(&self, mut channel: ssh2::Channel) {
        if let Err(error) = channel.send_eof() {
            log::error!("Sending EOF failed: {}", error);
        }
        else {
            if let Err(error) = channel.wait_eof() {
                log::error!("Waiting EOF failed: {}", error);
            }
        }

        if let Err(error) = channel.close() {
            log::error!("Error while closing channel: {}", error);
        }
        else {
            if let Err(error) = channel.wait_close() {
                log::error!("Error while closing channel: {}", error);
            }
        }
    }
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
            password: settings.get("password").map(|s| s.clone()),
            private_key_path: settings.get("private_key_path").map(|s| s.clone()),
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
            if let Err(error) = self.session.userauth_pubkey_file(self.username.as_str(), None, path, None) {
                return Err(format!("Failed to authenticate with private key: {}", error));
            };
        }
        else {
            log::warn!("Password is not set, trying authentication with first key found in SSH agent");
            if let Err(error) = self.session.userauth_agent(self.username.as_str()) {
                return Err(format!("Error when communicating with SSH agent: {}", error));
            };
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

    fn download_file(&self, source: &String) -> io::Result<(i32, Vec<u8>)> {
        let result: Result<(i32, Vec<u8>), io::Error>;

        match self.session.scp_recv(Path::new(&source)) {
            Ok((mut remote_file, stat)) => {
                let mut contents = Vec::new();
                if let Err(error) = remote_file.read_to_end(&mut contents) {
                    result = Err(error);
                }
                else {
                    result = Ok((stat.mode(), contents));
                }

                self.send_eof_and_close(remote_file);
            },
            Err(error) => {
                result = Err(io::Error::new(io::ErrorKind::Other, error.message()));
            }
        };

        result
    }

    fn upload_file(&self, metadata: &FileMetadata, contents: Vec<u8>) -> io::Result<()> {
        let result: io::Result<()>;

        match self.session.scp_send(Path::new(&metadata.remote_path), metadata.mode, contents.len().try_into().unwrap(), None) {
            Ok(mut remote_file) => {
                if let Err(error) = remote_file.write(&contents) {
                    result = Err(error)
                }
                else {
                    result = Ok(());
                }
                self.send_eof_and_close(remote_file);
            },
            Err(error) => {
                result = Err(io::Error::new(io::ErrorKind::Other, error.message()));
            }
        };

        result
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

