use std::collections::HashMap;
use crate::frontend;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("docker-inspect", "0.0.1")]
pub struct Inspect;

impl Module for Inspect {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Inspect { }
    }
}

impl CommandModule for Inspect {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-containers"),
            parent_id: String::from("docker-containers"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("search"),
            display_text: String::from("Inspect"),
            action: UIAction::TextView,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, parameters: Vec<String>) -> String {
        let target_id = parameters.first().expect("1 parameter is mandatory and should contain a container ID");

        // TODO: filter out all but alphanumeric characters
        format!("sudo curl --unix-socket /var/run/docker.sock http://localhost/containers/{}/json?all=true", target_id)
    }

    fn process_response(&self, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
}