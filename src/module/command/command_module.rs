
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
        Ok(CommandResult::new_info(response.message.clone()))
    }

    fn process_responses(&self, _host: Host, _responses: Vec<ResponseMessage>) -> Result<CommandResult, String> {
        Err(String::from("NI"))
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
    /// For commands with partial responses, as normally the response means the command is 100% done.
    /// This is percentage of progress (0-100).
    pub progress: u8,
    pub show_in_notification: bool,
    pub error: String,
    pub criticality: Criticality,
    pub time: DateTime<Utc>,
    pub invocation_id: u64,
}

impl CommandResult {
    pub fn new_hidden<Stringable: ToString>(message: Stringable) -> Self {
        CommandResult {
            message: message.to_string(),
            criticality: Criticality::Normal,
            show_in_notification: false,
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
            ..Default::default()
        }
    }

    pub fn new_warning<Stringable: ToString>(message: Stringable) -> Self {
        CommandResult {
            message: message.to_string(),
            criticality: Criticality::Warning,
            show_in_notification: true,
            ..Default::default()
        }
    }

    pub fn new_error<Stringable: ToString>(error: Stringable) -> Self {
        CommandResult {
            error: error.to_string(),
            criticality: Criticality::Error,
            show_in_notification: true,
            ..Default::default()
        }
    }

    pub fn new_critical_error<Stringable: ToString>(error: Stringable) -> Self {
        CommandResult {
            error: error.to_string(),
            criticality: Criticality::Critical,
            show_in_notification: true,
            ..Default::default()
        }
    }

    /// Used to inform that command is now queued but not yet in execution.
    pub fn pending(invocation_id: u64) -> Self {
        CommandResult {
            criticality: Criticality::NoData,
            invocation_id: invocation_id,
            progress: 0,
            ..Default::default()
        }
    }

    pub fn with_invocation_id(&mut self, invocation_id: u64) -> Self {
        self.invocation_id = invocation_id;
        self.to_owned()
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
            invocation_id: 0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UIAction {
    None,
    DetailsDialog,
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