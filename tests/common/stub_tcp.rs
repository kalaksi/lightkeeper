use std::collections::HashMap;

use lightkeeper::error::LkError;
use lightkeeper_module::connection_module;
use lightkeeper::module::*;
use lightkeeper::module::connection::*;

#[connection_module(
    name="tcp",
    version="0.0.1",
    description="Stub TCP",
    settings={
    }
)]
pub struct StubTcp {
    responses: HashMap<&'static str, ResponseMessage>,
}

impl Default for StubTcp {
    fn default() -> Self {
        StubTcp {
            responses: HashMap::new(),
        }
    }
}

impl StubTcp {
    pub fn new(address: &'static str, pem_certificate: &'static str) -> connection::Connector {
        let mut stub = StubTcp::default();
        stub.add_response(address, pem_certificate, 0);
        Box::new(stub) as connection::Connector
    }

    pub fn new_error(address: &'static str, error_message: &'static str) -> connection::Connector {
        let mut stub = StubTcp::default();
        stub.add_response(address, error_message, 1);
        Box::new(stub) as connection::Connector
    }

    pub fn new_any(pem_certificate: &'static str) -> connection::Connector {
        let mut stub = StubTcp::default();
        stub.add_response("_", pem_certificate, 0);
        Box::new(stub) as connection::Connector
    }

    pub fn add_response(&mut self, address: &'static str, response: &'static str, exit_code: i32) {
        self.responses.insert(address, ResponseMessage::new(response.to_string(), exit_code));
    }
}

impl Module for StubTcp {
    fn new(_settings: &HashMap<String, String>) -> Self {
        StubTcp::default()
    }
}

impl ConnectionModule for StubTcp {
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        let response = match self.responses.get(message) {
            Some(response) => response.clone(),
            None => {
                match self.responses.get("_") {
                    Some(response) => response.clone(),
                    None => return Err(LkError::other_p("No test response set up for address", message))
                }
            }
        };

        Ok(response)
    }
}

