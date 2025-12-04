/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use lightkeeper::error::LkError;
use lightkeeper_module::connection_module;
use lightkeeper::file_handler::FileMetadata;
use lightkeeper::module::*;
use lightkeeper::module::connection::*;


#[connection_module(
    name="ssh",
    version="0.0.1",
    description="Stub SSH",
    settings={
    }
)]
/// SSH connection module. Manages parallel SSH sessions internally.
pub struct StubSsh2 {
    responses: HashMap<&'static str, ResponseMessage>,
}

impl StubSsh2 {
    pub fn new(request: &'static str, response: &'static str, exit_code: i32) -> connection::Connector {
        let mut ssh = StubSsh2 {
            responses: HashMap::new(),
        };

        ssh.add_response(request, response, exit_code);
        Box::new(ssh) as connection::Connector
    }

    pub fn new_any(response: &'static str, exit_code: i32) -> connection::Connector {
        let mut ssh = StubSsh2 {
            responses: HashMap::new(),
        };

        ssh.add_response("_", response, exit_code);
        Box::new(ssh) as connection::Connector
    }

    pub fn add_response(&mut self, request: &'static str, response: &'static str, exit_code: i32) {
        self.responses.insert(request, ResponseMessage::new(response.to_string(), exit_code));
    }
}

impl Module for StubSsh2 {
    fn new(_settings: &HashMap<String, String>) -> Self {
        StubSsh2 {
            responses: HashMap::new(),
        }
    }
}

impl ConnectionModule for StubSsh2 {
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

    fn send_message_partial(&self, _message: &str, _invocation_id: u64) -> Result<ResponseMessage, LkError> {
        unimplemented!()
    }

    fn receive_partial_response(&self, _invocation_id: u64) -> Result<ResponseMessage, LkError> {
        unimplemented!()
    }

    fn download_file(&self, _source: &str) -> Result<(FileMetadata, Vec<u8>), LkError> {
        unimplemented!()
    }

    fn upload_file(&self, _metadata: &FileMetadata, _contents: Vec<u8>) -> Result<(), LkError> {
        unimplemented!()
    }

    fn verify_host_key(&self, _hostname: &str, _key_id: &str) -> Result<(), LkError> {
        unimplemented!()
    }
}

