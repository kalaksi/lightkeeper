use std::collections::HashMap;
use crate::enums::Criticality;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::string_validation;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module("systemd-service-start", "0.0.1")]
pub struct Start;

impl Module for Start {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Start { }
    }
}

impl CommandModule for Start {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("start"),
            display_text: String::from("Start"),
            // Only displayed if the service isn't running.
            depends_on_criticality: vec![Criticality::Error, Criticality::Critical],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let service = parameters.first().unwrap();
        if !string_validation::is_alphanumeric_with(service, "-_.@\\") ||
            string_validation::begins_with_dash(service){
            panic!("Invalid unit name: {}", service)
        }

        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if host.platform.is_newer_than(platform_info::Flavor::Debian, "8") {
                command.arguments(vec!["systemctl", "start", service]);
            }

            command.use_sudo = host.settings.contains(&HostSetting::UseSudo);
        }

        command.to_string()
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            Ok(CommandResult::new_error(response.message.clone()))
        }
        else {
            Ok(CommandResult::new(response.message.clone()))
        }
    }
}