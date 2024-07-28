use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
use lightkeeper_module::command_module;

#[command_module(
    name="systemd-service-unmask",
    version="0.0.1",
    description="Unmasks a SystemD service.",
)]
pub struct Unmask;

impl Module for Unmask {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Unmask { }
    }
}

impl CommandModule for Unmask {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("ok"),
            display_text: String::from("Unmask"),
            depends_on_tags: vec![String::from("masked")],
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

            command.arguments(vec!["systemctl", "unmask", service]);
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