use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
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
        let mut command = String::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command = match host.platform.os_flavor {
                Flavor::Debian => String::from("apt-get clean"),
                Flavor::Ubuntu => String::from("apt-get clean"),
                Flavor::RedHat => String::from("yum clean all"),
                _ => String::new(),
            };

            if !command.is_empty() && host.settings.contains(&HostSetting::UseSudo) {
                command = format!("sudo {}", command);
            }
        }

        command
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