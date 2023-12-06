use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::command::UIAction;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use chrono::NaiveDateTime;
use lightkeeper_module::command_module;

#[command_module(
    name="logs",
    version="0.0.1",
    description="Shows logs from journalctl.",
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
            tab_title: String::from("Host logs"),
            action: UIAction::LogViewWithTimeControls,
            ..Default::default()
        }
    }

    // Parameter 1 is for unit selection and special values "all" and "dmesg".
    // Parameter 2 is for grepping. Filters rows based on regexp.
    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let start_time = parameters.get(0).cloned().unwrap_or(String::from("-1h"));
        let end_time = parameters.get(1).cloned().unwrap_or(String::from("now"));
        let page_number = parameters.get(2).unwrap_or(&String::from("")).parse::<i32>().unwrap_or(-1);
        let page_size = parameters.get(3).unwrap_or(&String::from("")).parse::<i32>().unwrap_or(1000);

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "8") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "20") {
            // TODO: centos?

            command.arguments(vec!["journalctl", "-q"]);

            if !start_time.is_empty() {
                if start_time == "-1h" {
                    command.arguments(vec!["--since", &start_time]);
                }
                else {
                    match NaiveDateTime::parse_from_str(start_time.as_str(), "%Y-%m-%d %H:%M:%S") {
                        Ok(_) => command.arguments(vec!["--since", &start_time]),
                        Err(_) => return Err(format!("Invalid start time: {}", start_time)),
                    };
                }
            }
            if !end_time.is_empty() && end_time != "now" {
                match NaiveDateTime::parse_from_str(end_time.as_str(), "%Y-%m-%d %H:%M:%S") {
                    Ok(_) => command.arguments(vec!["--until", &end_time]),
                    Err(_) => return Err(format!("Invalid end time: {}", end_time)),
                };
            }

            if page_number > 0 {
                let row_count = page_number * page_size;
                command.arguments(vec!["-n", &row_count.to_string()]);
            }

            /* TODO: log searching on server side?
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
            } */

            Ok(command.to_string())
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_error() {
            return Err(response.message.clone());
        }
        Ok(CommandResult::new_hidden(response.message.clone()))
    }
}