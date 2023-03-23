use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
use lightkeeper_module::command_module;

#[command_module("docker-restart", "0.0.1")]
pub struct Restart;

impl Module for Restart {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Restart { }
    }
}

impl CommandModule for Restart {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-containers"),
            parent_id: String::from("docker-containers"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("refresh"),
            display_text: String::from("Restart"),
            confirmation_text: String::from("Really restart container?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> String {
        let mut command = ShellCommand::new();

        let target_id = parameters.first().unwrap();
        if !string_validation::is_alphanumeric(target_id) {
            panic!("Invalid container ID: {}", target_id)
        }

        if host.platform.os == platform_info::OperatingSystem::Linux {
            let url = format!("http://localhost/containers/{}/restart", target_id);
            command.arguments(vec!["curl", "--unix-socket", "/var/run/docker.sock", "-X", "POST", &url]);
            command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);
        }

        command.to_string()
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
}