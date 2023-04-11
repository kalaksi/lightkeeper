use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
use lightkeeper_module::command_module;

#[command_module("systemd-service-unmask", "0.0.1")]
pub struct Unmask;

impl Module for Unmask {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Unmask { }
    }
}

impl CommandModule for Unmask {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("ok"),
            display_text: String::from("unmask"),
            depends_on_tags: vec![String::from("masked")],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let service = parameters.first().unwrap();
        if !string_validation::is_alphanumeric_with(service, "-_.@\\") ||
            string_validation::begins_with_dash(service){
            panic!("Invalid unit name: {}", service)
        }

        let mut command = ShellCommand::new();
        command.arguments(vec!["systemctl", "unmask", service]);
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);
        command.to_string()
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            Ok(CommandResult::new_error(response.message.clone()))
        }
        else {
            Ok(CommandResult::new(response.message.clone()))
        }
    }
}