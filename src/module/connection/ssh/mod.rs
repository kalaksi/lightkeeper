use std::{
    net::TcpStream,
    net::IpAddr,
    net::Ipv4Addr,
    io::Read,
    collections::HashMap,
    path::Path,
    io,
    io::Write,
};

use ssh2::Session;
use crate::utils::strip_newline;
use lightkeeper_module::connection_module;
use crate::module::*;
use crate::module::connection::*;

#[connection_module("ssh", "0.0.1")]
pub struct Ssh2 {
    session: Session,
    is_initialized: bool,
    address: IpAddr,
    port: u16,
    username: String,
    password: Option<String>,
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
            port: 22,
            username: settings.get("username").unwrap().clone(),
            password: settings.get("password").map(|s| s.clone()),
        }
    }
}

impl ConnectionModule for Ssh2 {
    fn connect(&mut self, address: &IpAddr) -> Result<(), String> {
        if self.is_initialized {
            return Ok(())
        }

        self.address = address.clone();

        let stream = match TcpStream::connect(format!("{}:{}", address, self.port)) {
            Ok(stream) => stream,
            Err(error) => return Err(format!("Connection error: {}", error))
        };

        log::info!("Connected to {}:{}", address, self.port);

        self.session = Session::new().unwrap();
        self.session.set_tcp_stream(stream);
        if let Err(error) = self.session.handshake() {
            return Err(format!("Handshake error: {}", error));
        };

        if self.password.is_some() {
            if let Err(error) = self.session.userauth_password(self.username.as_str(), self.password.as_ref().unwrap().as_str()) {
                return Err(format!("Authentication error: {}", error));
            };
        }
        else {
            log::debug!("Password is not set, trying authentication with first key found in SSH agent");
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

        Ok(ResponseMessage {
            message: strip_newline(&output),
            return_code: exit_status,
        })
    }

    fn download_file(&self, source: &String) -> io::Result<Vec<u8>> {
        let result: Result<Vec<u8>, io::Error>;

        match self.session.scp_recv(Path::new(&source)) {
            Ok((mut remote_file, _)) => {
                let mut contents = Vec::new();
                if let Err(error) = remote_file.read_to_end(&mut contents) {
                    result = Err(error);
                }
                else {
                    result = Ok(contents);
                }

                self.send_eof_and_close(remote_file);
            },
            Err(error) => {
                result = Err(io::Error::new(io::ErrorKind::Other, error.message()));
            }
        };

        result
    }

    fn upload_file(&self, destination: &String, contents: Vec<u8>) -> io::Result<()> {
        let result: io::Result<()>;

        // TODO: keep original permissions.
        match self.session.scp_send(Path::new(destination), 0o640, contents.len().try_into().unwrap(), None) {
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

