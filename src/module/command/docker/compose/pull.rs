use regex::Regex;
use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="docker-compose-pull",
    version="0.0.1",
    description="Pulls images for docker-compose projects or services.",
)]
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
            depends_on_no_tags: vec![String::from("Local")],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let compose_file = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") {

            command.arguments(vec!["docker", "compose", "-f", compose_file, "pull"]);
            if let Some(service_name) = parameters.get(2) {
                command.argument(service_name);
            }
        }
        else {
            return Err(String::from("Unsupported platform"));
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &connection::ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_hidden(&response.message))
        } else {
            let mut errors = 0;
            // Check if the error is because it's a locally built image.
            for line in response.message.lines() {
                let errors_regex = Regex::new(r"\w*(\d+) errors? occurred").unwrap();
                if errors_regex.is_match(line) {
                    errors = errors_regex.captures(line).unwrap().get(1).unwrap().as_str().parse().unwrap_or_default();
                }
                // Locally built localhost/something-tagged images may result in this error.
                // Ignoring the error for now, but may mask real errors, in theory.
                else if line.contains("dial tcp 127.0.0.1:80: connect: connection refused") {
                    errors -= 1;
                }
            }

            if errors > 0 {
                Err(response.message.clone())
            }
            else {
                Ok(CommandResult::new_hidden(&response.message))
            }
        }
    }
}