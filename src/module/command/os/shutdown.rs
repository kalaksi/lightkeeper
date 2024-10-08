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
    name="shutdown",
    version="0.0.1",
    description="Shuts down the host.",
)]
pub struct Shutdown;

impl Module for Shutdown {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Shutdown { }
    }
}

impl CommandModule for Shutdown {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("shutdown"),
            display_text: String::from("Shut down"),
            confirmation_text: String::from("Really shut down host?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.argument("poweroff");
            Ok(command.to_string())
        }
        else {
            return Err(LkError::unsupported_platform());
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            Ok(CommandResult::new_warning(response.message.clone()))
        }
        else {
            Ok(CommandResult::new_info(response.message.clone()))
        }
    }
}