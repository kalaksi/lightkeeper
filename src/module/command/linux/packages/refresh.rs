
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
    name="linux-packages-refresh",
    version="0.0.1",
    description="Refreshes (or updates) package lists.",
)]
pub struct Refresh;

impl Module for Refresh {
    fn new(_settings: &HashMap<String, String>) -> Refresh {
        Refresh {
        }
    }
}

impl CommandModule for Refresh {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("packages"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("refresh"),
            display_text: String::from("Refresh package lists"),
            action: UIAction::FollowOutput,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") {
            command.arguments(vec!["apt", "update"]); 
        }
        else if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
                host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") {
            command.arguments(vec!["dnf", "check-update"]);
        }
        else {
            return Err(LkError::new_unsupported_platform());
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

impl Refresh {
    fn parse_progress_for_apt(&self, response: &ResponseMessage) -> u8 {
        for line in response.message.lines().rev() {
            if line.starts_with("Reading state information") {
                return 90;
            }
            else if line.starts_with("Reading package lists") {
                return 80;
            }
            else if line.starts_with("Hit:") {
                // Approximates the progress since the output doesn't tell the total amount.
                let hits = response.message.lines().filter(|line| line.starts_with("Hit:")).count();
                return std::cmp::min(70, hits / 2 * 10) as u8;
            }
        }

        0
    }
}