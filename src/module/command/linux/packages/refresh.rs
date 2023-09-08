use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="linux-packages-refresh",
    version="0.0.1",
    description="Refreshes (or updates) package lists.",
)]
pub struct Refresh;

impl Module for Refresh {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Self { }
    }
}

impl CommandModule for Refresh {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("packages"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("refresh"),
            display_text: String::from("Refresh package lists"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, String> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "9") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "20") {
            command.arguments(vec!["apt", "update"]); 
        }
        else if host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "8") ||
                host.platform.version_is_same_or_greater_than(platform_info::Flavor::RedHat, "8") {
            command.arguments(vec!["dnf", "check-update"]);
        }
        else {
            return Err(String::from("Unsupported platform"));
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        // TODO: view output messages of installation (can be pretty long)?
        if response.return_code == 0 {
            Ok(CommandResult::default())
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}