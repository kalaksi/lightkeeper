use std::{ net::TcpStream, net::IpAddr, io::Read };
use ssh2::Session;

use crate::module::{
    Module,
    metadata::Metadata,
    connection::ConnectionModule,
    connection::AuthenticationDetails,
    ModuleSpecification,
};


pub struct Ssh2 {
    session: Session,
    port: u16,
}

impl Module for Ssh2 {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from("ssh"), String::from("0.0.1")),
            display_name: String::from("SSH"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new() -> Self {
        // TODO: error handling?
        let session = Session::new().unwrap();

        Ssh2 {
            session: session,
            port: 22,
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }

}

impl ConnectionModule for Ssh2 {
    fn connect(&mut self, address: &IpAddr, authentication: Option<AuthenticationDetails>) -> Result<(), String> {
        // TODO: support ipv6

        let authentication = authentication.unwrap_or_default();

        if !authentication.use_authentication {
            return Err(String::from("Enabling authentication is required for SSH"));
        };


        let stream = match TcpStream::connect(format!("{}:{}", address, self.port)) {
            Ok(stream) => stream,
            Err(error) => return Err(format!("Connection error: {}", error))
        };

        log::info!("Connected to {}", address);

        self.session.set_tcp_stream(stream);
        if let Err(error) = self.session.handshake() {
            return Err(format!("Handshake error: {}", error));
        };

        if let Err(error) = self.session.userauth_agent(authentication.username.as_str()) {
            return Err(format!("Error when communicating with authentication agent: {}", error));
        };

        Ok(())
    }

    fn send_message(&self, message: &str) -> Result<String, String>
    {
        // TODO: more elegant error system

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

        if let Err(error) = channel.wait_close() {
            log::error!("Error while closing channel: {}", error);
        };

        Ok(output)
    }

    fn is_connected(&self) -> bool {
        // TODO: better way for this (there wasn't anything obvious)
        match self.session.banner() {
            Some(_) => true,
            None => false,
        }
    }

}

