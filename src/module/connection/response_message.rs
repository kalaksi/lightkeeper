
#[derive(Debug, Clone)]
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
}