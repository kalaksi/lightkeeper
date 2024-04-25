
use std::collections::HashMap;
use crate::error::LkError;
use crate::module::MetadataSupport;
use crate::module::module::Module;
use crate::module::connection::ResponseMessage;
use crate::file_handler::FileMetadata;

pub type Connector = Box<dyn ConnectionModule + Send + Sync>;

pub trait ConnectionModule : MetadataSupport + Module {
    fn set_target(&self, _address: &str) {
    }

    /// Sends a request / message and waits for response. Response can be complete or partial.
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError>;

    fn send_message_partial(&self, _message: &str, _invocation_id: u64) -> Result<ResponseMessage, LkError> {
        panic!("Not implemented");
    }

    /// For partial responses. Should be called until the response is complete.
    fn receive_partial_response(&self, _invocation_id: u64) -> Result<ResponseMessage, LkError> {
        panic!("Not implemented");
    }

    fn download_file(&self, _source: &String) -> Result<(FileMetadata, Vec<u8>), LkError> {
        Err(LkError::new_other("Not implemented"))
    }

    fn upload_file(&self, _metadata: &FileMetadata, _contents: Vec<u8>) -> Result<(), LkError> {
        Err(LkError::new_other("Not implemented"))
    }

    fn verify_host_key(&self, _hostname: &str, _key_id: &str) -> Result<(), LkError> {
        Err(LkError::new_other("Not implemented"))
    }

    /// Check the connection status. Only relevant to modules that use a persistent connection.
    fn is_connected(&self) -> bool {
        true
    }

    fn new_connection_module(settings: &HashMap<String, String>) -> Connector where Self: Sized + 'static + Send + Sync {
        Box::new(Self::new(settings))
    }
}