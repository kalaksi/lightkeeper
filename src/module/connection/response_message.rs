
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub message: String,
    pub return_code: i32,
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