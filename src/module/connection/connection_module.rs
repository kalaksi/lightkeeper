/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use crate::error::LkError;
use crate::file_handler::FileMetadata;
use crate::module::connection::ResponseMessage;
use crate::module::module::Module;
use crate::module::MetadataSupport;

pub type Connector = Box<dyn ConnectionModule + Send + Sync>;

pub trait ConnectionModule: BoxCloneableConnector + MetadataSupport + Module {
    /// Stores target address. Should be called before anything else since connects/reconnects can happen at any point.
    fn set_target(&self, _address: &str) {}

    /// Sends a request / message and waits for response. Response can be complete or partial.
    fn send_message(&self, message: &str) -> Result<ResponseMessage, LkError>;

    fn send_message_partial(&self, _message: &str, _invocation_id: u64) -> Result<ResponseMessage, LkError> {
        Err(LkError::not_implemented())
    }

    /// For partial responses. Should be called until the response is complete.
    fn receive_partial_response(&self, _invocation_id: u64) -> Result<ResponseMessage, LkError> {
        Err(LkError::not_implemented())
    }

    fn download_file(&self, _source: &str) -> Result<(FileMetadata, Vec<u8>), LkError> {
        Err(LkError::not_implemented())
    }

    fn upload_file(&self, _metadata: &FileMetadata, _contents: Vec<u8>) -> Result<(), LkError> {
        Err(LkError::not_implemented())
    }

    /// Sends a command and writes data to its stdin, then reads the response.
    fn send_message_with_stdin(&self, _command: &str, _stdin_data: &[u8]) -> Result<ResponseMessage, LkError> {
        Err(LkError::not_implemented())
    }

    /// Sends a command and reads binary output (for file downloads).
    fn send_message_binary(&self, _command: &str) -> Result<(Vec<u8>, i32), LkError> {
        Err(LkError::not_implemented())
    }

    fn verify_host_key(&self, _hostname: &str, _key_id: &str) -> Result<(), LkError> {
        Err(LkError::not_implemented())
    }

    fn new_connection_module(settings: &HashMap<String, String>) -> Connector
    where
        Self: Sized + 'static + Send + Sync,
    {
        Box::new(Self::new(settings))
    }
}

// Implemented by the macro.
pub trait BoxCloneableConnector {
    fn box_clone(&self) -> Connector;
}
