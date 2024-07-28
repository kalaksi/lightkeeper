use regex::Regex;
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
    name="linux-packages-update-all",
    version="0.0.1",
    description="Updates all system packages.",
)]
pub struct UpdateAll {
    regex_install_counts: Regex,
}

impl Module for UpdateAll {
    fn new(_settings: &HashMap<String, String>) -> UpdateAll {
        UpdateAll {
            regex_install_counts: Regex::new(r"(?i)\w*(\d+) upgraded, (\d+) newly installed").unwrap(),
        }
    }
}

impl CommandModule for UpdateAll {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("packages"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("update"),
            display_text: String::from("Upgrade all packages"),
            confirmation_text: String::from("Really upgrade all packages?"),
            action: UIAction::FollowOutput,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") {
            command.arguments(vec!["apt", "upgrade", "-y"]); 
        }
        else if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
                host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") {
            command.arguments(vec!["dnf", "update", "-y"]); 
        }
        else {
            return Err(LkError::unsupported_platform());
        }
        Ok(command.to_string())
    }

    fn process_response(&self, host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            let progress = if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
                              host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") {
                self.parse_progress_for_apt(response)
            }
            else if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
                    host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") {
                1
            }
            else {
                panic!()
            };

            Ok(CommandResult::new_partial(response.message.clone(), progress))
        }
        else {
            if response.return_code == 0 {
                Ok(CommandResult::new_hidden(response.message.clone()))
            }
            else {
                Ok(CommandResult::new_hidden(response.message.clone())
                                 .with_criticality(crate::enums::Criticality::Error))
            }
        }
    }
}

impl UpdateAll {
    // It's not critical if apt output format changes.
    // It will only make the progress reporting less granular.
    fn parse_progress_for_apt(&self, response: &ResponseMessage) -> u8 {
        let mut total_to_install: u32 = 0;

        for line in response.message.lines() {
            if let Some(captures) = self.regex_install_counts.captures(line) {
                let upgraded = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
                let new = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
                total_to_install = upgraded + new;
                break;
            }
        }

        if total_to_install > 0 {
            let unpack_count = response.message.lines().filter(|line| line.contains("Preparing to unpack ")).count() as u32;
            let unpack_progress = (unpack_count * 100 / total_to_install) as u8;

            let setting_up_count = response.message.lines().filter(|line| line.contains("Setting up ")).count() as u32;
            let setting_up_progress = (setting_up_count * 100 / total_to_install) as u8;

            10 + (unpack_progress as f32 * 0.4) as u8 + (setting_up_progress as f32 * 0.4) as u8
        }
        else if response.message.contains("Reading state information") {
            10
        }
        else {
            0
        }
    }
}