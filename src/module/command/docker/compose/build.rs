use regex::Regex;

use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;




#[command_module(
    name="docker-compose-build",
    version="0.0.1",
    description="Builds local docker-compose service images.",
)]
pub struct Build {
    regex_step: Regex,
}

impl Module for Build {
    fn new(_settings: &HashMap<String, String>) -> Build {
        Build {
            regex_step: Regex::new(r"(?i)\w*step (\d+)/(\d+)").unwrap(),
        }
    }
}

impl CommandModule for Build {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-compose"),
            parent_id: String::from("docker-compose"),
            action: UIAction::FollowOutput,
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("build"),
            display_text: String::from("Build"),
            depends_on_tags: vec![String::from("Local")],
            multivalue_level: 2,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let compose_file = parameters.first().unwrap();
        let service_name = parameters.get(2).unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") {

            command.arguments(vec!["docker", "compose", "-f", compose_file, "build", service_name]);
        }
        else {
            return Err(LkError::unsupported_platform())
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &connection::ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            let mut progress: u8 = 1;
            for line in response.message.lines().rev() {
                if let Some(captures) = self.regex_step.captures(line) {
                    let current = captures.get(1).unwrap().as_str().parse::<u8>().unwrap();
                    let total = captures.get(2).unwrap().as_str().parse::<u8>().unwrap();
                    progress = (current as f32 / total as f32 * 100.0) as u8;
                    break;
                }
            }

            Ok(CommandResult::new_partial(response.message.clone(), progress))
        }
        else {
            if response.return_code == 0 {
                Ok(CommandResult::new_hidden(response.message.clone()))
            } else {
                Ok(CommandResult::new_hidden(response.message.clone())
                                 .with_criticality(crate::enums::Criticality::Error))
            }
        }
    }
}