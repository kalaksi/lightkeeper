/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::process::{self, Stdio};
use std::sync::{Arc, Mutex};

use lightkeeper_module::stateless_connection_module;
use crate::error::LkError;
use crate::module::*;
use crate::module::connection::*;

const PARTIAL_READ_TIMEOUT_MS: i32 = 2000;

#[stateless_connection_module(
    name="local-command",
    version="0.0.1",
    description="Executes a command locally.",
)]
pub struct LocalCommand {
    open_process: Arc<Mutex<Option<OpenProcess>>>,
}

struct OpenProcess {
    child: process::Child,
    stdout: process::ChildStdout,
}

impl Module for LocalCommand {
    fn new(_settings: &HashMap<String, String>) -> Self {
        LocalCommand {
            open_process: Arc::new(Mutex::new(None)),
        }
    }
}

impl ConnectionModule for LocalCommand {
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError> {
        let output = process::Command::new("bash")
            .args(["-c", message])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(ResponseMessage::new_success(stdout))
        }
        else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(ResponseMessage::new(stderr, output.status.code().unwrap_or(1)))
        }
    }

    fn send_message_partial(&self, message: &str, _invocation_id: u64) -> Result<ResponseMessage, LkError> {
        let merged_command = format!("{{ {}; }} 2>&1", message);
        let mut child = process::Command::new("bash")
            .args(["-c", &merged_command])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let mut stdout = child.stdout.take()
            .ok_or_else(|| LkError::other("Failed to capture stdout"))?;

        let (output, is_eof) = Self::poll_read(&mut stdout)?;
        if is_eof {
            let exit_status = child.wait()
                .map(|s| s.code().unwrap_or(-1))
                .unwrap_or(-1);
            Ok(ResponseMessage::new_partial_complete(output, exit_status))
        }
        else {
            *self.open_process.lock()? = Some(OpenProcess { child, stdout });
            Ok(ResponseMessage::new_partial(output))
        }
    }

    fn receive_partial_response(&self, _invocation_id: u64) -> Result<ResponseMessage, LkError> {
        let mut open = self.open_process.lock()?.take()
            .ok_or_else(|| LkError::other("No open process for partial response"))?;

        match Self::poll_read(&mut open.stdout) {
            Ok((output, true)) => {
                let exit_status = open.child.wait()
                    .map(|s| s.code().unwrap_or(-1))
                    .unwrap_or(-1);

                Ok(ResponseMessage::new_partial_complete(output, exit_status))
            }
            Ok((output, false)) => {
                *self.open_process.lock()? = Some(open);
                Ok(ResponseMessage::new_partial(output))
            }
            Err(e) => {
                let _ = open.child.kill();
                let _ = open.child.wait();
                Err(e)
            }
        }
    }

    fn interrupt(&self, _invocation_id: u64) -> Result<(), LkError> {
        if let Some(ref mut open) = *self.open_process.lock()? {
            let _ = open.child.kill();
        }
        Ok(())
    }
}

impl LocalCommand {
    /// Waits up to `PARTIAL_READ_TIMEOUT_MS` for data on stdout.
    /// Returns (output, is_eof).
    fn poll_read(stdout: &mut process::ChildStdout) -> Result<(String, bool), LkError> {
        let fd = stdout.as_raw_fd();
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ret = unsafe { libc::poll(&mut pollfd, 1, PARTIAL_READ_TIMEOUT_MS) };
        if ret < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if ret == 0 {
            return Ok((String::new(), false));
        }

        let mut buffer = [0u8; 4096];
        let bytes_read = stdout.read(&mut buffer)?;

        if bytes_read == 0 {
            Ok((String::new(), true))
        }
        else {
            Ok((String::from_utf8_lossy(&buffer[..bytes_read]).to_string(), false))
        }
    }
}
