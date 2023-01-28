use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("reboot", "0.0.1")]
pub struct Reboot;

impl Module for Reboot {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Reboot { }
    }
}

impl CommandModule for Reboot {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("refresh"),
            display_text: String::from("Reboot"),
            confirmation_text: String::from("Really reboot host?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> String {
        let mut command = String::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command = String::from("reboot");

            if host.settings.contains(&HostSetting::UseSudo) {
                command = format!("sudo {}", command);
            }
        }

        command
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            Ok(CommandResult::new_warning(response.message.clone()))
        }
        else {
            Ok(CommandResult::new(response.message.clone()))
        }
    }
}