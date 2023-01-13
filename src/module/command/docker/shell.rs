use std::collections::HashMap;
use crate::frontend;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("docker-shell", "0.0.1")]
pub struct Shell;

impl Module for Shell {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Shell { }
    }
}

impl CommandModule for Shell {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-containers"),
            parent_id: String::from("docker-containers"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("terminal"),
            display_text: String::from("Open shell inside"),
            action: UIAction::Terminal,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _platform: PlatformInfo, parameters: Vec<String>) -> String {
        // TODO: filter out all but alphanumeric characters
        let target_id = parameters.first().expect("1 parameter is mandatory and should contain a container ID");

        // TODO
        String::new()
    }

    fn process_response(&self, _platform: PlatformInfo, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
}