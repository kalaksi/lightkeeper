use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
use lightkeeper_module::command_module;

#[command_module(
    name="systemd-service-logs",
    version="0.0.1",
    description="Shows journald logs of a systemd service.",
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
            category: String::from("systemd"),
            parent_id: String::from("systemd-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("view-document"),
            display_text: String::from("Show logs"),
            action: UIAction::LogView,
            tab_title: String::from("Service logs"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let service = parameters.get(0).unwrap();
        let start_time = parameters.get(1).cloned().unwrap_or(String::from(""));
        let end_time = parameters.get(2).cloned().unwrap_or(String::from(""));
        let page_number = parameters.get(3).unwrap_or(&String::from("")).parse::<i32>().unwrap_or(1);
        let page_size = parameters.get(4).unwrap_or(&String::from("")).parse::<i32>().unwrap_or(1000);

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if !string_validation::is_alphanumeric_with(service, "-_.@\\") ||
            string_validation::begins_with_dash(service){

            return Err(LkError::other_p("Invalid unit name", service));
        }

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            command.arguments(vec!["journalctl", "-q", "-u", service]);

            if !start_time.is_empty() {
                command.arguments(vec!["--since", &start_time]);
            }
            if !end_time.is_empty() {
                command.arguments(vec!["--until", &end_time]);
            }

            if page_number > 0 {
                let row_count = page_number * page_size;
                command.arguments(vec!["-n", &row_count.to_string()]);
            }

            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_error() {
            Err(response.message.clone())
        }
        else {
            Ok(CommandResult::new_hidden(response.message.clone()))
        }
    }
}