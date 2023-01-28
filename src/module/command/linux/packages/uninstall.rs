use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module("linux-packages-uninstall", "0.0.1")]
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
        Some(ModuleSpecification::new("ssh", "0.0.1"))
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

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> String {
        let mut command = String::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if host.platform.os_flavor == platform_info::Flavor::Debian ||
               host.platform.os_flavor == platform_info::Flavor::Ubuntu {

                command = String::from("apt-get remove");

                if self.purge {
                    command = format!("{} --purge", command);
                }
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