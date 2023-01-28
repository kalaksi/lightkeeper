use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("docker-restart", "0.0.1")]
pub struct Restart;

impl Module for Restart {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Restart { }
    }
}

impl CommandModule for Restart {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-containers"),
            parent_id: String::from("docker-containers"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("refresh"),
            display_text: String::from("Restart"),
            confirmation_text: String::from("Really restart container?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let mut command = String::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            // TODO: filter out all but alphanumeric characters
            let target_id = parameters.first().expect("1 parameter is mandatory and should contain a container ID");
            command = format!("curl --unix-socket /var/run/docker.sock -X POST http://localhost/containers/{}/restart", target_id);

            if host.settings.contains(&HostSetting::UseSudo) {
                command = format!("sudo {}", command);
            }
        }

        command
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
}