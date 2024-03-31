
use std::collections::HashMap;
use crate::error::LkError;
use crate::module::MetadataSupport;
use crate::module::module::Module;
use crate::module::connection::ResponseMessage;
use crate::file_handler::FileMetadata;

pub type Connector = Box<dyn ConnectionModule + Send>;

pub trait ConnectionModule : MetadataSupport + Module {
    /// Sends a request / message and waits for response. Response can be complete or partial.
    fn send_message(&mut self, message: &str, wait_full_response: bool) -> Result<ResponseMessage, LkError>;

    /// For partial responses. Should be called until the response is complete.
    fn receive_partial_response(&mut self) -> Result<ResponseMessage, LkError> {
        panic!("Not implemented");
    }

    fn download_file(&self, _source: &String) -> Result<(FileMetadata, Vec<u8>), LkError> {
        Err(LkError::new_other("Not implemented"))
    }

    fn upload_file(&self, _metadata: &FileMetadata, _contents: Vec<u8>) -> Result<(), LkError> {
        Err(LkError::new_other("Not implemented"))
    }

    fn verify_host_key(&mut self, _hostname: &str, _key_id: &str) -> Result<(), LkError> {
        Err(LkError::new_other("Not implemented"))
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
    fn connect(&mut self, _address: &str) -> Result<(), LkError> {
        Ok(())
    }

    fn disconnect(&mut self) {
        // Do nothing by default.
    }

    fn reconnect(&mut self) -> Result<(), LkError> {
        Ok(())
    }
}