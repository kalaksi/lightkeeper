
use std::collections::HashMap;
use serde_derive::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    module::Module,
    module::ModuleSpecification,
    module::MetadataSupport,
    module::connection::ResponseMessage,
    enums::Criticality,
    frontend,
    host::Host,
};

pub type Command = Box<dyn CommandModule + Send + Sync>;

pub trait CommandModule : BoxCloneableCommand + MetadataSupport + Module {
    fn new_command_module(settings: &HashMap<String, String>) -> Command where Self: Sized + 'static + Send + Sync {
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

    fn get_connector_message(&self, _host: Host, _parameters: Vec<String>) -> Result<String, String> {
        Err(String::new())
    }

    fn get_connector_messages(&self, _host: Host, _parameters: Vec<String>) -> Result<Vec<String>, String> {
        Err(String::new())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
    fn process_responses(&self, _host: Host, _responses: Vec<ResponseMessage>) -> Result<CommandResult, String> {
        Err(String::from("Not implemented"))
    }

    fn error_unsupported(&self) -> Result<CommandResult, String> {
        Err(String::from("Unsupported platform"))
    }
}

// Implemented by the macro.
pub trait BoxCloneableCommand {
    fn box_clone(&self) -> Command;
}


#[derive(Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command_id: String,
    pub message: String,
    pub error: String,
    pub criticality: Criticality,
    pub time: DateTime<Utc>,
    pub invocation_id: u64,
}

impl CommandResult {
    pub fn new(message: String) -> Self {
        CommandResult {
            message: message,
            criticality: Criticality::Normal,
            ..Default::default()
        }
    }

    pub fn new_info(message: String) -> Self {
        CommandResult {
            message: message,
            criticality: Criticality::Info,
            ..Default::default()
        }
    }

    pub fn new_warning(message: String) -> Self {
        CommandResult {
            message: message,
            criticality: Criticality::Warning,
            ..Default::default()
        }
    }

    pub fn new_error(error: String) -> Self {
        CommandResult {
            error: error,
            criticality: Criticality::Error,
            ..Default::default()
        }
    }

    pub fn new_critical_error(error: String) -> Self {
        CommandResult {
            error: error,
            criticality: Criticality::Critical,
            ..Default::default()
        }
    }
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult {
            command_id: String::from(""),
            message: String::from(""),
            error: String::from(""),
            criticality: Criticality::Normal,
            time: Utc::now(),
            invocation_id: 0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum UIAction {
    None,
    DetailsDialog,
    TextView,
    TextDialog,
    LogView,
    Terminal,
    TextEditor,
}

impl Default for UIAction {
    fn default() -> Self {
        UIAction::None
    }
}