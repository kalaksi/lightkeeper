use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_json;

use crate::frontend;
use crate::module::connection::ResponseMessage;
use crate::utils::enums::Criticality;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("docker-image-remove", "0.0.1")]
pub struct Remove;

impl Module for Remove {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Remove { }
    }
}

impl CommandModule for Remove {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-images"),
            parent_id: String::from("docker-images"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("delete"),
            display_text: String::from("Delete"),
            confirmation_text: String::from("Really remove image?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, parameters: Vec<String>) -> String {
        // TODO: validate target_id
        let target_id = parameters.first().unwrap();
        format!("sudo curl --unix-socket /var/run/docker.sock -X DELETE http://localhost/images/{}", target_id)
    }

    fn process_response(&self, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            let docker_response: JsonMessage = serde_json::from_str(&response.message).unwrap();
            return Ok(CommandResult::new_with_level(docker_response.message, Criticality::Error));
        }
        Ok(CommandResult::new(response.message.clone()))
    }
}

#[derive(Deserialize)]
struct JsonMessage {
    message: String,
}
