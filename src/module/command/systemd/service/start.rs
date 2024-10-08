use std::collections::HashMap;
use crate::enums::Criticality;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::string_validation;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="systemd-service-start",
    version="0.0.1",
    description="Starts a SystemD service.",
)]
pub struct Start;

impl Module for Start {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Start { }
    }
}

impl CommandModule for Start {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
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

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let service = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if !string_validation::is_alphanumeric_with(service, "-_.@\\") ||
            string_validation::begins_with_dash(service){

            Err(LkError::other_p("Invalid unit name", service))
        }
        else if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
            host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
            host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
            host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
            host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            command.arguments(vec!["systemctl", "start", service]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if !response.message.is_empty() {
            Ok(CommandResult::new_error(response.message.clone()))
        }
        else {
            Ok(CommandResult::new_info(response.message.clone()))
        }
    }
}