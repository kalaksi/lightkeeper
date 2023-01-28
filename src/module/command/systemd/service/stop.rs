use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("systemd-service-stop", "0.0.1")]
pub struct Stop;

impl Module for Stop {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Stop { }
    }
}

impl CommandModule for Stop {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("stop"),
            display_text: String::from("Stop"),
            confirmation_text: String::from("Really stop service?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let mut command = String::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if host.platform.is_newer_than(platform_info::Flavor::Debian, "8") {
                let service = parameters.first().unwrap();
                command = format!("sudo systemctl stop {}", service);
            }

            if !command.is_empty() && host.settings.contains(&HostSetting::UseSudo) {
                command = format!("sudo {}", command);
            }
        }

        command
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