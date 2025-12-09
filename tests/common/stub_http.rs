/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use lightkeeper::error::LkError;
use lightkeeper_module::stateless_connection_module;
use lightkeeper::module::*;
use lightkeeper::module::connection::*;

#[stateless_connection_module(
    name="http",
    version="0.0.1",
    description="Stub HTTP",
    settings={
    }
)]
pub struct StubHttp {
    // Responses are stored in shared state since HTTP is stateless
    // This field is kept for the struct but not used
    _unused: (),
}

// Shared state for all StubHttp instances (since HTTP is stateless and creates new instances)
fn get_shared_responses() -> &'static Mutex<HashMap<&'static str, ResponseMessage>> {
    static RESPONSES: OnceLock<Mutex<HashMap<&'static str, ResponseMessage>>> = OnceLock::new();
    RESPONSES.get_or_init(|| Mutex::new(HashMap::new()))
}

impl Default for StubHttp {
    fn default() -> Self {
        StubHttp {
            _unused: (),
        }
    }
}

impl StubHttp {
    pub fn clear_responses() {
        let mut shared = get_shared_responses().lock().unwrap();
        shared.clear();
    }

    pub fn new(url: &'static str, response: &'static str) -> connection::Connector {
        let mut shared = get_shared_responses().lock().unwrap();
        shared.insert(url, ResponseMessage::new(response.to_string(), 0));
        drop(shared);
        Box::new(StubHttp::default()) as connection::Connector
    }

    pub fn new_error(url: &'static str, error_message: &'static str) -> connection::Connector {
        let mut shared = get_shared_responses().lock().unwrap();
        shared.insert(url, ResponseMessage::new(error_message.to_string(), 1));
        drop(shared);
        Box::new(StubHttp::default()) as connection::Connector
    }

    pub fn new_any(response: &'static str) -> connection::Connector {
        let mut shared = get_shared_responses().lock().unwrap();
        shared.insert("_", ResponseMessage::new(response.to_string(), 0));
        drop(shared);
        Box::new(StubHttp::default()) as connection::Connector
    }

    pub fn add_response(&mut self, url: &'static str, response: &'static str, exit_code: i32) {
        let mut shared = get_shared_responses().lock().unwrap();
        shared.insert(url, ResponseMessage::new(response.to_string(), exit_code));
    }
}

impl Module for StubHttp {
    fn new(_settings: &HashMap<String, String>) -> Self {
        StubHttp::default()
    }
}

impl ConnectionModule for StubHttp {
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        if message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        // HTTP connector sends URL on first line, data on second line (if POST)
        // Use split("\n") to match the actual HTTP connector behavior
        let url = message.split("\n").next().unwrap_or("_").trim();
        
        // Use shared responses since HTTP is stateless and creates new instances
        let shared = get_shared_responses().lock().unwrap();
        
        // Try exact match first
        if let Some(response) = shared.get(url) {
            return Ok(response.clone());
        }
        
        // Try wildcard match
        if let Some(response) = shared.get("_") {
            return Ok(response.clone());
        }
        
        // Try partial matching - check if URL contains any key or vice versa
        for (key, resp) in shared.iter() {
            if url == *key || url.contains(*key) || key.contains(url) {
                return Ok(resp.clone());
            }
        }
        drop(shared);
        Err(LkError::other_p("No test response set up for URL", message))
    }
}

