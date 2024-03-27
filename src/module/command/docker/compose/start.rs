use std::collections::HashMap;
use crate::enums::Criticality;
use crate::frontend;
use crate::host::*;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="docker-compose-start",
    version="0.0.1",
    description="Starts docker-compose projects or services.",
)]
pub struct Start {
}

impl Module for Start {
    fn new(_settings: &HashMap<String, String>) -> Start {
        Start {
        }
    }
}

impl CommandModule for Start {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-compose"),
            parent_id: String::from("docker-compose"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("start"),
            display_text: String::from("Start"),
            // Only displayed if the service isn't running.
            depends_on_criticality: vec![Criticality::Error, Criticality::Critical],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let compose_file = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") {

            command.arguments(vec!["docker-compose", "-f", compose_file, "start"]);
            if let Some(service_name) = parameters.get(2) {
                command.argument(service_name);
            }
        }
        else if host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
                host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") {

            command.arguments(vec!["docker", "compose", "-f", compose_file, "start"]);
            if let Some(service_name) = parameters.get(2) {
                command.argument(service_name);
            }
        }
        else {
            return Err(String::from("Unsupported platform"));
        }
        Ok(command.to_string())
    }
}