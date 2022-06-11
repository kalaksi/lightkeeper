
use std::collections::HashMap;
use std::net::IpAddr;
use crate::module::module::Module;

pub type Connector = Box<dyn ConnectionModule + Send>;

pub trait ConnectionModule : Module {
    fn connect(&mut self, address: &IpAddr) -> Result<(), String>;

    // Send message over the established connection.
    fn send_message(&self, message: &str) -> Result<String, String>;

    // Check the connection status. Only relevant to modules that use a persistent connection.
    fn is_connected(&self) -> bool {
        false
    }

    fn new_connection_module(settings: &HashMap<String, String>) -> Connector where Self: Sized + 'static + Send {
        Box::new(Self::new(settings))
    }
}