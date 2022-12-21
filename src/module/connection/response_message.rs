
#[derive(Debug, Clone)]
pub struct ResponseMessage {
    pub message: String,
    pub return_code: i32,
}

impl ResponseMessage {
    pub fn empty() -> ResponseMessage {
        ResponseMessage {
            message: String::new(),
            return_code: 0,
        }
    }
}