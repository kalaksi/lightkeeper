use crate::{enums::Criticality, error::LkError};
use serde_derive::{ Serialize, Deserialize };


#[derive(Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub message: String,
    pub criticality: Criticality,
}

impl From<LkError> for ErrorMessage {
    fn from(error: LkError) -> Self {
        ErrorMessage {
            message: error.to_string(),
            criticality: Criticality::Error,
        }
    }
}