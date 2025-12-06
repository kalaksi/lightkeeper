/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use crate::{
    enums::Criticality,
    error::LkError,
    frontend,
    host::Host,
    module::{connection::ResponseMessage, MetadataSupport, Module, ModuleSpecification},
};

pub type Command = Box<dyn CommandModule + Send + Sync>;

pub trait CommandModule: BoxCloneableCommand + MetadataSupport + Module {
    fn new_command_module(settings: &HashMap<String, String>) -> Command
    where
        Self: Sized + 'static + Send + Sync,
    {
        Box::new(Self::new(settings))
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        None
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: self.get_module_spec().id,
            ..Default::default()
        }
    }

    /// Should never panic.
    fn get_connector_message(&self, _host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        Err(LkError::not_implemented())
    }

    /// Should never panic.
    fn get_connector_messages(&self, _host: Host, _parameters: Vec<String>) -> Result<Vec<String>, LkError> {
        Err(LkError::not_implemented())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new_info(response.message.clone()))
    }

    fn process_responses(&self, _host: Host, _responses: Vec<ResponseMessage>) -> Result<CommandResult, LkError> {
        Err(LkError::not_implemented())
    }
}

// Implemented by the macro.
pub trait BoxCloneableCommand {
    fn box_clone(&self) -> Command;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub command_id: String,
    pub message: String,
    /// For commands with partial responses, as normally the response means the command is 100% done.
    /// This is percentage of progress (0-100).
    pub progress: u8,
    pub show_in_notification: bool,
    pub error: String,
    pub criticality: Criticality,
    pub time: DateTime<Utc>,
}

impl CommandResult {
    pub fn new_hidden<Stringable: ToString>(message: Stringable) -> Self {
        CommandResult {
            message: message.to_string(),
            criticality: Criticality::Normal,
            show_in_notification: false,
            progress: 100,
            ..Default::default()
        }
    }

    pub fn new_partial<Stringable: ToString>(message: Stringable, progress: u8) -> Self {
        CommandResult {
            message: message.to_string(),
            criticality: Criticality::Normal,
            show_in_notification: false,
            progress: progress,
            ..Default::default()
        }
    }

    pub fn new_info<Stringable: ToString>(message: Stringable) -> Self {
        CommandResult {
            message: message.to_string(),
            criticality: Criticality::Info,
            show_in_notification: true,
            progress: 100,
            ..Default::default()
        }
    }

    pub fn new_warning<Stringable: ToString>(message: Stringable) -> Self {
        CommandResult {
            message: message.to_string(),
            criticality: Criticality::Warning,
            show_in_notification: true,
            progress: 100,
            ..Default::default()
        }
    }

    pub fn new_error<Stringable: ToString>(error: Stringable) -> Self {
        CommandResult {
            error: error.to_string(),
            criticality: Criticality::Error,
            show_in_notification: true,
            progress: 100,
            ..Default::default()
        }
    }

    pub fn new_critical_error<Stringable: ToString>(error: Stringable) -> Self {
        CommandResult {
            error: error.to_string(),
            criticality: Criticality::Critical,
            show_in_notification: true,
            progress: 100,
            ..Default::default()
        }
    }

    /// Used to inform that command is now queued but not yet in execution.
    pub fn pending() -> Self {
        CommandResult {
            criticality: Criticality::NoData,
            progress: 0,
            ..Default::default()
        }
    }

    pub fn with_criticality(&mut self, criticality: Criticality) -> Self {
        self.criticality = criticality;
        self.to_owned()
    }
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult {
            command_id: String::from(""),
            message: String::from(""),
            progress: 100,
            show_in_notification: false,
            error: String::from(""),
            criticality: Criticality::Normal,
            time: Utc::now(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UIAction {
    None,
    TextView,
    TextDialog,
    LogView,
    LogViewWithTimeControls,
    Terminal,
    TextEditor,
    FollowOutput,
}

impl Default for UIAction {
    fn default() -> Self {
        UIAction::None
    }
}
