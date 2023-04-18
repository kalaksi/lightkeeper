
use std::collections::HashMap;
use std::net::IpAddr;
use std::io;
use crate::module::MetadataSupport;
use crate::module::module::Module;
use crate::module::connection::ResponseMessage;

pub type Connector = Box<dyn ConnectionModule + Send>;

pub trait ConnectionModule : MetadataSupport + Module {
    /// Send message over the established connection.
    fn send_message(&mut self, message: &str) -> Result<ResponseMessage, String>;

    fn download_file(&self, _source: &String) -> io::Result<Vec<u8>> {
        Err(io::Error::new(io::ErrorKind::Other, "Not implemented"))
    }

    fn upload_file(&self, _destination: &String, _contents: Vec<u8>) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, "Not implemented"))
    }

    /// Check the connection status. Only relevant to modules that use a persistent connection.
    fn is_connected(&self) -> bool {
        false
    }

    fn new_connection_module(settings: &HashMap<String, String>) -> Connector where Self: Sized + 'static + Send {
        Box::new(Self::new(settings))
    }

    // These are only relevant to modules that use a persistent connection.

    /// Connect to the specified address. Should do nothing if already connected.
    fn connect(&mut self, _address: &IpAddr) -> Result<(), String> {
        Ok(())
    }

    fn disconnect(&mut self) {
        // Do nothing by default.
    }

    fn reconnect(&mut self) -> Result<(), String> {
        Ok(())
    }
}