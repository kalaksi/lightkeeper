use std::{
    collections::HashMap,
};
use crate::frontend;
use crate::host::*;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module("docker-compose-pull", "0.0.1")]
pub struct Pull {
}

impl Module for Pull {
    fn new(_settings: &HashMap<String, String>) -> Pull {
        Pull {
        }
    }
}

impl CommandModule for Pull {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-compose"),
            parent_id: String::from("docker-compose"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("download"),
            display_text: String::from("Pull"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            let compose_file = parameters.first().unwrap();
            command.arguments(vec!["docker-compose", "-f", compose_file, "pull"]);

            if let Some(service_name) = parameters.get(1) {
                command.argument(service_name);
            }

            command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);
            Ok(command.to_string())
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: &connection::ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::default())
        } else {
            Err(response.message.clone())
        }
    }
}