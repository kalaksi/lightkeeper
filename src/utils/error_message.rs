use crate::enums::Criticality;
use serde_derive::{ Serialize, Deserialize };


#[derive(Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub message: String,
    pub criticality: Criticality,
}

impl ErrorMessage {
    pub fn new(criticality: Criticality, message: String) -> Self {
        ErrorMessage {
            message,
            criticality,
        }
    }
}