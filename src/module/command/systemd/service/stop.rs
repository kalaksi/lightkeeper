use std::collections::HashMap;
use crate::frontend;
use crate::host::Host;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("systemd-service-stop", "0.0.1")]
pub struct Stop;

impl Module for Stop {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Stop { }
    }
}

impl CommandModule for Stop {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("stop"),
            display_text: String::from("Stop"),
            confirmation_text: String::from("Really stop service?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _host: Host, parameters: Vec<String>) -> String {
        let service = parameters[0].clone();
        let mut command = format!("sudo systemctl stop {}", service);
        command
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            Ok(CommandResult::new_error(response.message.clone()))
        }
        else {
            Ok(CommandResult::new(response.message.clone()))
        }
    }
}