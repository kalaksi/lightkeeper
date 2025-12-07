/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use lightkeeper::error::LkError;
use lightkeeper_module::stateless_connection_module;
use lightkeeper::module::*;
use lightkeeper::module::connection::*;


#[stateless_connection_module(
    name="local-command",
    version="0.0.1",
    description="Stub Local Command",
    settings={
    }
)]
/// Local command connection module stub for testing.
pub struct StubLocalCommand {
    responses: HashMap<&'static str, ResponseMessage>,
}

impl StubLocalCommand {
    pub fn new(request: &'static str, response: &'static str, exit_code: i32) -> connection::Connector {
        let mut stub = StubLocalCommand {
            responses: HashMap::new(),
        };

        stub.add_response(request, response, exit_code);
        Box::new(stub) as connection::Connector
    }

    pub fn new_any(response: &'static str, exit_code: i32) -> connection::Connector {
        let mut stub = StubLocalCommand {
            responses: HashMap::new(),
        };

        stub.add_response("_", response, exit_code);
        Box::new(stub) as connection::Connector
    }

    pub fn add_response(&mut self, request: &'static str, response: &'static str, exit_code: i32) {
        self.responses.insert(request, ResponseMessage::new(response.to_string(), exit_code));
    }
}

impl Module for StubLocalCommand {
    fn new(_settings: &HashMap<String, String>) -> Self {
        StubLocalCommand {
            responses: HashMap::new(),
        }
    }
}

impl ConnectionModule for StubLocalCommand {
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        let response = match self.responses.get(message) {
            Some(response) => response.clone(),
            None => {
                match self.responses.get("_") {
                    Some(response) => response.clone(),
                    None => return Err(LkError::other_p("No test response set up for command", message))
                }
            }
        };

        Ok(response)
    }
}

