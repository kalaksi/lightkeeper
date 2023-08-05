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

#[command_module(
    name="docker-image-remove",
    version="0.0.1",
    description="Removes a Docker image.",
)]
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
            parent_id: String::from("docker-image-updates"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("delete"),
            display_text: String::from("Delete"),
            confirmation_text: String::from("Really remove image?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let target_id = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if !string_validation::is_alphanumeric_with(target_id, ":-.") {
            panic!("Invalid image ID: {}", target_id)
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            let url = format!("http://localhost/images/{}", target_id);
            command.arguments(vec!["curl", "--unix-socket", "/var/run/docker.sock", "-X", "DELETE", &url]);
            Ok(command.to_string())
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            if let Ok(deletion_details) = serde_json::from_str::<Vec<DeletionDetails>>(&response.message) {
                let response_message = deletion_details.iter().map(|details| {
                    if let Some(deleted) = &details.deleted {
                        format!("Deleted: {}", deleted)
                    }
                    else if let Some(untagged) = &details.untagged {
                        format!("Untagged: {}", untagged)
                    }
                    else {
                        String::from("")
                    }
                }).collect::<Vec<String>>().join("\n");

                return Ok(CommandResult::new(response_message));
            }
            else if let Ok(docker_response) = serde_json::from_str::<ErrorMessage>(&response.message) {
                return Ok(CommandResult::new_error(docker_response.message));
            }
        }
        Ok(CommandResult::new(response.message.clone()))
    }
}

#[derive(Deserialize)]
struct DeletionDetails {
    untagged: Option<String>,
    deleted: Option<String>,
}

#[derive(Deserialize)]
struct ErrorMessage {
    message: String,
}
