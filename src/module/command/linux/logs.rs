use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::command::UIAction;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::string_validation;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    "logs",
    "0.0.1",
    "Shows logs from journalctl.
    Settings: none"
)]
pub struct Logs;

impl Module for Logs {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Logs { }
    }
}

impl CommandModule for Logs {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("view-document"),
            display_text: String::from("Show logs"),
            action: UIAction::LogView,
            ..Default::default()
        }
    }

    // Parameter 1 is for unit selection and special values "all" and "dmesg".
    // Parameter 2 is for grepping. Filters rows based on regexp.
    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["journalctl", "-q", "-n", "400"]);

            if let Some(parameter1) = parameters.first() {
                if !parameter1.is_empty() {
                    if !string_validation::is_alphanumeric_with(parameter1, "-_.@\\") ||
                        string_validation::begins_with_dash(parameter1){
                        panic!("Invalid unit name: {}", parameter1)
                    }

                    if parameter1 == "all" {
                        // No parameter needed.
                    } else if parameter1 == "dmesg" {
                        command.argument("--dmesg");
                    } else {
                        command.arguments(vec!["-u", parameter1]);
                    }
                }
            }

            if let Some(parameter2) = parameters.get(1) {
                if !parameter2.is_empty() {
                    command.arguments(vec!["-g", parameter2]);
                }
            }

            Ok(command.to_string())
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
}