/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::command_handler::CommandButtonData;
use crate::configuration::CustomCommandConfig;
use crate::frontend::frontend::VerificationRequest;
use crate::frontend::{DisplayData, HostDisplayData};

pub const PROTOCOL_VERSION: u16 = 2;
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Connect {
        protocol_version: u16,
    },
    ExecuteCommand {
        request_id: u64,
        host_id: String,
        command_id: String,
        parameters: Vec<String>,
    },
    CommandsForHost {
        request_id: u64,
        host_id: String,
    },
    CommandForHost {
        request_id: u64,
        host_id: String,
        command_id: String,
    },
    CustomCommandsForHost {
        request_id: u64,
        host_id: String,
    },
    AllHostCategories {
        request_id: u64,
        host_id: String,
    },
    VerifyHostKey {
        host_id: String,
        connector_id: String,
        key_id: String,
    },
    Disconnect,
    InterruptInvocation {
        invocation_id: u64,
    },
    RefreshHostMonitors {
        host_id: String,
    },
    RefreshPlatformInfo {
        host_id: String,
    },
    RefreshPlatformInfoAll {
        request_id: u64,
    },
    RefreshMonitorsForCommand {
        request_id: u64,
        host_id: String,
        command_id: String,
    },
    RefreshMonitorsOfCategory {
        request_id: u64,
        host_id: String,
        category: String,
    },
    RefreshCertificateMonitors {
        request_id: u64,
    },
    ResolveTextEditorPath {
        request_id: u64,
        host_id: String,
        command_id: String,
        parameters: Vec<String>,
    },
    DownloadEditableFile {
        request_id: u64,
        host_id: String,
        command_id: String,
        remote_file_path: String,
    },
    UploadEditedFile {
        request_id: u64,
        host_id: String,
        command_id: String,
        remote_file_path: String,
        contents: Vec<u8>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Connect {
        protocol_version: u16,
    },
    ExecuteCommand {
        request_id: u64,
        invocation_id: u64,
    },
    CommandsForHost {
        request_id: u64,
        host_id: String,
        commands: HashMap<String, CommandButtonData>,
    },
    CommandForHost {
        request_id: u64,
        host_id: String,
        command_id: String,
        command: Option<CommandButtonData>,
    },
    CustomCommandsForHost {
        request_id: u64,
        host_id: String,
        commands: HashMap<String, CustomCommandConfig>,
    },
    AllHostCategories {
        request_id: u64,
        host_id: String,
        categories: Vec<String>,
    },
    InitialState(DisplayData),
    HostUpdate(HostDisplayData),
    VerificationRequest(VerificationRequest),
    Error {
        request_id: Option<u64>,
        message: String,
    },
    RefreshInvocationIds {
        request_id: u64,
        invocation_ids: Vec<u64>,
    },
    InitializeHostsResult {
        request_id: u64,
        host_ids: Vec<String>,
    },
    ResolveTextEditorPath {
        request_id: u64,
        path: Option<String>,
    },
    DownloadEditableFileResult {
        request_id: u64,
        invocation_id: u64,
    },
    UploadEditedFileResult {
        request_id: u64,
        invocation_id: u64,
    },
}

pub fn read_message<T: DeserializeOwned, Reader: Read>(reader: &mut Reader) -> io::Result<T> {
    let mut length_buffer = [0_u8; 4];
    reader.read_exact(&mut length_buffer)?;

    let message_length = u32::from_be_bytes(length_buffer) as usize;
    if message_length == 0 || message_length > MAX_FRAME_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid frame length"));
    }

    let mut message_buffer = vec![0_u8; message_length];
    reader.read_exact(&mut message_buffer)?;

    bincode::deserialize(&message_buffer)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))
}

pub fn write_message<T: Serialize, Writer: Write>(writer: &mut Writer, message: &T) -> io::Result<()> {
    let serialized = bincode::serialize(message)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;

    if serialized.len() > MAX_FRAME_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Message is too large"));
    }

    writer.write_all(&(serialized.len() as u32).to_be_bytes())?;
    writer.write_all(&serialized)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framed_message_roundtrip_works() {
        let mut buffer = Vec::new();
        let message = ClientMessage::Connect {
            protocol_version: PROTOCOL_VERSION,
        };

        write_message(&mut buffer, &message).unwrap();

        let decoded: ClientMessage = read_message(&mut buffer.as_slice()).unwrap();
        match decoded {
            ClientMessage::Connect { protocol_version } => assert_eq!(protocol_version, PROTOCOL_VERSION),
            _ => panic!("Invalid message"),
        }
    }

    #[test]
    fn server_message_roundtrip_works() {
        let mut buffer = Vec::new();
        let message = ServerMessage::ExecuteCommand {
            request_id: 7,
            invocation_id: 42,
        };

        write_message(&mut buffer, &message).unwrap();

        let decoded: ServerMessage = read_message(&mut buffer.as_slice()).unwrap();
        match decoded {
            ServerMessage::ExecuteCommand {
                request_id,
                invocation_id,
            } => {
                assert_eq!(request_id, 7);
                assert_eq!(invocation_id, 42);
            }
            _ => panic!("Invalid message"),
        }
    }
}
