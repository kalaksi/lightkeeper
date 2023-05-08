use std::collections::HashMap;
use ureq;

use lightkeeper_module::stateless_connection_module;
use crate::module::*;
use crate::module::connection::*;

#[stateless_connection_module("http", "0.0.1", "Global")]
pub struct Http {
}

impl Http {
}

impl Module for Http {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Http {
        }
    }
}

impl ConnectionModule for Http {
    fn send_message(&mut self, message: &str) -> Result<ResponseMessage, String> {
        if message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let mut parts = message.split("\n");
        let url = parts.next().unwrap();
        let data = parts.next().unwrap_or_default();

        // Currently only supports GET and POST requests.
        let response = if data.is_empty() {
            ureq::get(url).call().map_err(|error| format!("Error while sending GET request: {}", error))?
        } else {
            ureq::post(url).send_string(data).map_err(|error| format!("Error while sending POST request: {}", error))?
        };

        let response_string = response.into_string().map_err(|error| format!("Error while reading response: {}", error))?;

        Ok(ResponseMessage {
            message: response_string,
            return_code: 0,
        })
    }


}