use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="linux-packages-uninstall",
    version="0.0.1",
    description="Uninstalls system packages.",
)]
pub struct Uninstall {
    pub purge: bool
}

impl Module for Uninstall {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Self {
            purge: true,
        }
    }
}

impl CommandModule for Uninstall {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("packages"),
            parent_id: String::from("package"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("delete"),
            display_text: String::from("Uninstall package"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let package = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") {
            command.arguments(vec!["apt-get", "remove", "-y", package]);

            if self.purge {
                command.argument("--purge");
            }
        }
        else if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
                host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") {
            command.arguments(vec!["dnf", "remove", "-y", package]);
        }
        else {
            return Err(LkError::unsupported_platform());
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code >= 0 {
            Ok(CommandResult::new_info(String::new()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}