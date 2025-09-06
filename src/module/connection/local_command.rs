/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::process;

use lightkeeper_module::stateless_connection_module;
use crate::error::LkError;
use crate::module::*;
use crate::module::connection::*;

#[stateless_connection_module(
    name="local-command",
    version="0.0.1",
    description="Executes a command locally.",
)]
pub struct LocalCommand {
}

impl LocalCommand {
}

impl Module for LocalCommand {
    fn new(_settings: &HashMap<String, String>) -> Self {
        LocalCommand {
        }
    }
}

impl ConnectionModule for LocalCommand {
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        // TODO: don't assume bash exists even though it's very common?
        let output = process::Command::new("bash")
                                      .args(&["-c", message])
                                      .output()
                                      .map_err(|e| e.to_string())?;

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).unwrap();
            Ok(ResponseMessage::new_success(stdout))
        }
        else {
            let stderr = String::from_utf8(output.stderr).unwrap();
            Ok(ResponseMessage::new(stderr, output.status.code().unwrap_or(1)))
        }
    }
}