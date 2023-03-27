use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_json;

use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
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

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let target_id = parameters.first().unwrap();
        if !string_validation::is_alphanumeric_with(target_id, ":-.") {
            panic!("Invalid image ID: {}", target_id)
        }

        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            let url = format!("http://localhost/images/{}/remove", target_id);
            command.arguments(vec!["curl", "--unix-socket", "/var/run/docker.sock", "-X", "DELETE", &url]);
            command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);
        }

        command.to_string()
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            let docker_response: JsonMessage = serde_json::from_str(&response.message).unwrap();
            return Ok(CommandResult::new_error(docker_response.message));
        }
        Ok(CommandResult::new(response.message.clone()))
    }
}

#[derive(Deserialize)]
struct JsonMessage {
    message: String,
}
