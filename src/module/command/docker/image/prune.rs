use std::collections::HashMap;
use serde_derive::Deserialize;
use serde_json;
use crate::frontend;
use crate::host::Host;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("docker-image-prune", "0.0.1")]
pub struct Prune;

impl Module for Prune {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Prune { }
    }
}

impl CommandModule for Prune {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-images"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("clear"),
            display_text: String::from("Prune"),
            confirmation_text: String::from("Really prune all unused images?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _host: Host, _parameters: Vec<String>) -> String {
        String::from("sudo curl --unix-socket /var/run/docker.sock -X POST http://localhost/images/prune")
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        let result: PruneResult = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;
        Ok(CommandResult::new_info(format!("Total reclaimed space: {} B", result.space_reclaimed)))
    }
}


#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PruneResult {
    // images_deleted: Option<Vec<String>>,
    space_reclaimed: i64,
}
