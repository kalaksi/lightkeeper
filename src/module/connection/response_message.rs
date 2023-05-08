use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub message: String,
    pub return_code: i32,
}

impl ResponseMessage {
    pub fn new(message: String) -> ResponseMessage {
        ResponseMessage {
            message: message,
            return_code: 0,
        }
    }

    pub fn empty() -> ResponseMessage {
        ResponseMessage {
            message: String::new(),
            return_code: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.message.is_empty() && self.return_code == 0
    }

    pub fn is_success(&self) -> bool {
        self.return_code == 0
    }
}