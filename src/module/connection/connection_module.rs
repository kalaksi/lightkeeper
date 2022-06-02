use std::net::IpAddr;
use crate::module::{
    module::Module,
    connection::Credentials,
};

pub trait ConnectionModule : Module {
    fn connect(&mut self, address: &IpAddr, authentication: Option<Credentials>) -> Result<(), String>;

    // Send message over the established connection.
    fn send_message(&self, message: &str) -> Result<String, String>;

    // Check the connection status. Only relevant to modules that use a persistent connection.
    fn is_connected(&self) -> bool {
        false
    }

    fn new_connection_module() -> Box<dyn ConnectionModule> where Self: Sized + 'static {
        Box::new(Self::new())
    }
}