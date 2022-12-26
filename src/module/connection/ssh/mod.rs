use std::{
    net::TcpStream,
    net::IpAddr,
    io::Read,
    collections::HashMap,
    path::Path,
    io,
    io::Write,
};

use ssh2::Session;
use crate::utils::strip_newline;
use crate::module::{
    Module,
    metadata::Metadata,
    connection::ConnectionModule,
    connection::ResponseMessage,
    ModuleSpecification,
};


pub struct Ssh2 {
    session: Session,
    is_initialized: bool,
    port: u16,
    username: String,
    // password: String,
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
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("ssh", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        // TODO: error handling?
        let session = Session::new().unwrap();

        Ssh2 {
            session: session,
            is_initialized: false,
            port: 22,
            username: settings.get("username").unwrap_or(&String::from("")).clone(),
            // password: settings.get("password").unwrap_or(&String::from("")).clone(),
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }

}

impl ConnectionModule for Ssh2 {
    fn connect(&mut self, address: &IpAddr) -> Result<(), String> {
        // TODO: support ipv6
        // TODO: reconnect when server side has disconnected

        if self.is_initialized {
            return Ok(())
        }

        let stream = match TcpStream::connect(format!("{}:{}", address, self.port)) {
            Ok(stream) => stream,
            Err(error) => return Err(format!("Connection error: {}", error))
        };

        log::info!("Connected to {}", address);

        self.session.set_tcp_stream(stream);
        if let Err(error) = self.session.handshake() {
            return Err(format!("Handshake error: {}", error));
        };

        if let Err(error) = self.session.userauth_agent(self.username.as_str()) {
            return Err(format!("Error when communicating with authentication agent: {}", error));
        };

        self.is_initialized = true;
        Ok(())
    }

    fn send_message(&self, message: &str) -> Result<ResponseMessage, String>
    {
        if message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let mut channel = match self.session.channel_session() {
            Ok(channel) => channel,
            Err(error) => return Err(format!("Error opening'{}'", error))
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
        if !self.is_initialized {
            return false;
        }

        // There didn't seem to be a better way to check the connection status.
        match self.session.banner_bytes() {
            Some(_) => true,
            None => false,
        }
    }

}

