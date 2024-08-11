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
    name="linux-packages-logs",
    version="0.0.1",
    description="Shows package manager logs",
)]
pub struct Logs;

impl Module for Logs {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Self { }
    }
}

impl CommandModule for Logs {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("packages"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("view-document"),
            display_text: String::from("Show logs"),
            tab_title: String::from("Package manager logs"),
            action: UIAction::LogView,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        // let start_time = parameters.get(0).cloned().unwrap_or(String::from("-1h"));
        // let end_time = parameters.get(1).cloned().unwrap_or(String::from("now"));
        let page_number = parameters.get(2).unwrap_or(&String::from("")).parse::<i32>().unwrap_or(-1);
        let page_size = parameters.get(3).unwrap_or(&String::from("")).parse::<i32>().unwrap_or(1000);

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        let row_count = if page_number > 0 {
            page_number * page_size
        }
        else {
            page_size
        };

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") {
            command.arguments(vec!["tail", "-n", &row_count.to_string(), "/var/log/apt/term.log"]);
        }
        else {
            return Err(LkError::unsupported_platform())
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_error() {
            return Err(response.message.clone());
        }
        Ok(CommandResult::new_hidden(response.message.clone()))
    }
}