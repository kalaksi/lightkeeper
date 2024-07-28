use std::collections::HashMap;
use serde_derive::Deserialize;
use serde_json;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="docker-image-prune",
    version="0.0.1",
    description="Prunes all unused Docker images.",
)]
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

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["curl", "-s", "--unix-socket", "/var/run/docker.sock", "-X", "POST", "http://localhost/images/prune"]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
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
