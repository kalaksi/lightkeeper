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

pub const PROTOCOL_VERSION: u16 = 8;
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
    WriteCachedFile {
        request_id: u64,
        host_id: String,
        remote_file_path: String,
        contents: Vec<u8>,
    },
    RemoveCachedFile {
        request_id: u64,
        host_id: String,
        remote_file_path: String,
    },
    HasCachedFileChanged {
        request_id: u64,
        host_id: String,
        remote_file_path: String,
        content_hash: String,
    },
    UploadFileFromCache {
        request_id: u64,
        host_id: String,
        command_id: String,
        remote_file_path: String,
    },
    GetConfig {
        request_id: u64,
    },
    UpdateConfig {
        request_id: u64,
        main_yml: String,
        hosts_yml: String,
        groups_yml: String,
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
    WriteCachedFileResult {
        request_id: u64,
    },
    RemoveCachedFileResult {
        request_id: u64,
    },
    HasCachedFileChangedResult {
        request_id: u64,
        changed: bool,
    },
    UploadFileFromCacheResult {
        request_id: u64,
        invocation_id: u64,
    },
    Config {
        request_id: u64,
        main_yml: String,
        hosts_yml: String,
        groups_yml: String,
    },
    UpdateConfigOk {
        request_id: u64,
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
