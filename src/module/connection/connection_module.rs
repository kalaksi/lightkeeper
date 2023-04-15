
use std::collections::HashMap;
use std::net::IpAddr;
use std::io;
use crate::module::MetadataSupport;
use crate::module::module::Module;
use crate::module::connection::ResponseMessage;

pub type Connector = Box<dyn ConnectionModule + Send>;

pub trait ConnectionModule : MetadataSupport + Module {
    /// Connect to the specified address. Should do nothing if already connected.
    fn connect(&mut self, address: &IpAddr) -> Result<(), String>;

    /// Send message over the established connection.
    fn send_message(&mut self, message: &str) -> Result<ResponseMessage, String>;

    fn download_file(&self, source: &String) -> io::Result<Vec<u8>>;
    fn upload_file(&self, destination: &String, contents: Vec<u8>) -> io::Result<()>;

    /// Check the connection status. Only relevant to modules that use a persistent connection.
    fn is_connected(&self) -> bool {
        false
    }

    fn disconnect(&mut self);

    fn reconnect(&mut self) -> Result<(), String>;

    fn new_connection_module(settings: &HashMap<String, String>) -> Connector where Self: Sized + 'static + Send {
        Box::new(Self::new(settings))
    }
}