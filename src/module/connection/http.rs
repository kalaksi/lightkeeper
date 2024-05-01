use std::collections::HashMap;
use ureq;

use lightkeeper_module::stateless_connection_module;
use crate::error::LkError;
use crate::module::*;
use crate::module::connection::*;

#[stateless_connection_module(
    name="http",
    version="0.0.1",
    cache_scope="Global",
    description="Sends a HTTP request",
)]
pub struct Http {
    // A temporary state for resource reuse when receiving multiple commands.
    agent: ureq::Agent,
}

impl Http {
}

impl Module for Http {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Http {
            agent: ureq::Agent::new(),
        }
    }
}

impl ConnectionModule for Http {
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        if message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let mut parts = message.split("\n");
        let url = parts.next().unwrap();
        let data = parts.next().unwrap_or_default();

        // Currently only supports GET and POST requests.
        let response = if data.is_empty() {
            self.agent.get(url).call().map_err(|error| format!("Error while sending GET request: {}", error))?
        } else {
            self.agent.post(url).send_string(data).map_err(|error| format!("Error while sending POST request: {}", error))?
        };

        let response_string = response.into_string().map_err(|error| format!("Error while reading response: {}", error))?;

        Ok(ResponseMessage::new_success(response_string))
    }


}