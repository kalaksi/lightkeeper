use std::collections::HashMap;
use crate::frontend;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("systemd-service-start", "0.0.1")]
pub struct Start;

impl Module for Start {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Start { }
    }
}

impl CommandModule for Start {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("start"),
            display_text: String::from("Start"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _platform: PlatformInfo, parameters: Vec<String>) -> String {
        let service = parameters[0].clone();
        let mut command = format!("sudo systemctl start {}", service);
        command
    }

    fn process_response(&self, _platform: PlatformInfo, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            Ok(CommandResult::new_error(response.message.clone()))
        }
        else {
            Ok(CommandResult::new(response.message.clone()))
        }
    }
}