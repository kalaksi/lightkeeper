use serde_derive::{Serialize, Deserialize};

use crate::{connection_manager::ConnectorRequest, host::Host};


#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RequestResponse {
    pub source_id: String,
    pub host: Host,
    pub invocation_id: u64,
    pub responses: Vec<Result<ResponseMessage, String>>,
}

impl RequestResponse {
    pub fn new(request: &ConnectorRequest, responses: Vec<Result<ResponseMessage, String>>) -> RequestResponse {
        RequestResponse {
            source_id: request.source_id.clone(),
            host: request.host.clone(),
            invocation_id: request.invocation_id.clone(),
            responses: responses,
            ..Default::default()
        }
    }

    pub fn new_empty(source_id: String, host: Host, invocation_id: u64) -> RequestResponse {
        RequestResponse {
            source_id: source_id,
            host: host,
            invocation_id: invocation_id,
            ..Default::default()
        }
    }

    pub fn new_error(source_id: String, host: Host, invocation_id: u64, error: String) -> RequestResponse {
        RequestResponse {
            source_id: source_id,
            host: host,
            invocation_id: invocation_id,
            responses: vec![Err(error)],
        }
    }
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub message: String,
    pub return_code: i32,
    pub is_partial: bool,
    pub is_from_cache: bool,
    /// Not found in cache when OnlyCache policy was used.
    pub not_found: bool,
}

impl ResponseMessage {
    pub fn new(message: String, return_code: i32) -> ResponseMessage {
        ResponseMessage {
            message: message,
            return_code: return_code,
            ..Default::default()
        }
    }

    pub fn new_partial(partial_message: String) -> ResponseMessage {
        ResponseMessage {
            message: partial_message,
            return_code: 0,
            is_partial: true,
            ..Default::default()
        }
    }

    pub fn new_success(message: String) -> ResponseMessage {
        ResponseMessage {
            message: message,
            ..Default::default()
        }
    }

    pub fn empty() -> ResponseMessage {
        ResponseMessage {
            ..Default::default()
        }
    }

    pub fn not_found() -> ResponseMessage {
        ResponseMessage {
            not_found: true,
            ..Default::default()
        }
    }

    pub fn is_success(&self) -> bool {
        self.return_code == 0
    }

    pub fn is_error(&self) -> bool {
        self.return_code != 0
    }

    pub fn is_empty(&self) -> bool {
        self.message.is_empty() && self.return_code == 0
    }

    pub fn is_not_found(&self) -> bool {
        self.not_found
    }
}