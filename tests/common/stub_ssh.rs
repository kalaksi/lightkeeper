/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use lightkeeper::error::LkError;
use lightkeeper::module::platform_info::*;
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
    pub fn new(flavor: Flavor) -> Self {
        let mut responses = HashMap::new();
        
        match flavor {
            Flavor::Debian => {
                responses.insert("cat /etc/os-release", ResponseMessage::new_success(
r#"PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
NAME="Debian GNU/Linux"
VERSION_ID="12"
VERSION="12 (bookworm)"
VERSION_CODENAME=bookworm
ID=debian
HOME_URL="https://www.debian.org/"
SUPPORT_URL="https://www.debian.org/support"
BUG_REPORT_URL="https://bugs.debian.org/""#));
                responses.insert("uname -m", ResponseMessage::new_success("x86_64"));

            },
            _ => unimplemented!(),
        };

        StubSsh2 {
            responses
        }
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
        let response = self.responses.get(message)
            .expect("Response not defined")
            .clone();

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

