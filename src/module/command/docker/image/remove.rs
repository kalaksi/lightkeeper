use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_json;

use crate::frontend;
use crate::utils::enums::Criticality;
use crate::module::{
    Module,
    connection::ResponseMessage,
    command::CommandModule,
    command::Command,
    command::CommandResult,
    Metadata,
    ModuleSpecification,
};


#[derive(Clone)]
pub struct Remove;

impl Module for Remove {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-image-remove", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Remove { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Remove {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-images"),
            parent_id: String::from("docker-images"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("delete"),
            display_priority: 2,
            confirmation_text: String::from("Really remove image?"),
            ..Default::default()
        }
    }

    fn get_connector_request(&self, target_id: String) -> String {
        // TODO: validate target_id
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
