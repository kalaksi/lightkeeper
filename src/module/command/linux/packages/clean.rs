use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::module::platform_info::Flavor;
use lightkeeper_module::command_module;

#[command_module("linux-packages-clean", "0.0.1")]
pub struct Clean;

impl Module for Clean {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Self { }
    }
}

impl CommandModule for Clean {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("packages"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("clear"),
            display_text: String::from("Clean package cache"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> String {
        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            match host.platform.os_flavor {
                Flavor::Debian => command.arguments(vec!["apt-get", "clean"]),
                Flavor::Ubuntu => command.arguments(vec!["apt-get", "clean"]),
                Flavor::RedHat => command.arguments(vec!["yum", "clean", "all"]),
                _ => (),
            };

            command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);
        }

        command.to_string()
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code >= 0 {
            Ok(CommandResult::new(String::new()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}