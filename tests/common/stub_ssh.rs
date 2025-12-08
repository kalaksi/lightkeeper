/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lightkeeper::error::LkError;
use lightkeeper::utils::strip_newline;
use lightkeeper_module::connection_module;
use lightkeeper::module::*;
use lightkeeper::module::connection::*;


const DEFAULT_PARTIAL_MESSAGE_SIZE: usize = 20;

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
    partial_message_size: usize,
    partial_responses: Arc<Mutex<HashMap<u64, ResponseMessage>>>,
}

impl StubSsh2 {
    pub fn new(request: &'static str, response: &'static str, exit_code: i32) -> connection::Connector {
        let mut ssh = StubSsh2::default();
        ssh.add_response(request, response, exit_code);
        Box::new(ssh) as connection::Connector
    }

    pub fn new_any(response: &'static str, exit_code: i32) -> connection::Connector {
        let mut ssh = StubSsh2::default();
        ssh.add_response("_", response, exit_code);
        Box::new(ssh) as connection::Connector
    }

    pub fn add_response(&mut self, request: &'static str, response: &'static str, exit_code: i32) {
        self.responses.insert(request, ResponseMessage::new(response.to_string(), exit_code));
    }
}

impl Default for StubSsh2 {
    fn default() -> Self {
        StubSsh2 {
            responses: HashMap::new(),
            partial_message_size: DEFAULT_PARTIAL_MESSAGE_SIZE,
            partial_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Module for StubSsh2 {
    fn new(_settings: &HashMap<String, String>) -> Self {
        StubSsh2 {
            responses: HashMap::new(),
            partial_message_size: 20,
            partial_responses: Arc::new(Mutex::new(HashMap::new())),
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

    fn send_message_partial(&self, message: &str, invocation_id: u64) -> Result<ResponseMessage, LkError> {
        let response = match self.responses.get(message) {
            Some(response) => response.clone(),
            None => {
                match self.responses.get("_") {
                    Some(response) => response.clone(),
                    None => return Err(LkError::other_p("No test response set up for command", message))
                }
            }
        };

        if response.message.len() > self.partial_message_size {
            let response_message = response.message.clone();
            let (partial_message, remaining_message) = response_message.split_at(self.partial_message_size);
            self.partial_responses.lock().unwrap().insert(
                invocation_id,
                ResponseMessage::new(remaining_message.to_string(), response.return_code)
            );

            return Ok(ResponseMessage::new_partial(partial_message.to_string()))
        }
        else {
            Ok(ResponseMessage::new(strip_newline(&response.message), response.return_code))
        }
    }

    fn receive_partial_response(&self, invocation_id: u64) -> Result<ResponseMessage, LkError> {
        let mut partial_responses = self.partial_responses.lock().unwrap();
        let partial_response = partial_responses.get_mut(&invocation_id).unwrap();

        if partial_response.message.len() > self.partial_message_size {
            let current_message = partial_response.message.clone();
            let (response, remaining_message) = current_message.split_at(self.partial_message_size);
            partial_response.message = remaining_message.to_string();
            return Ok(ResponseMessage::new_partial(response.to_string()));
        }
        else {
            let response = partial_response.message.to_string();
            self.partial_responses.lock().unwrap().remove(&invocation_id);
            return Ok(ResponseMessage::new(strip_newline(&response), 0));
        }
    }
}

