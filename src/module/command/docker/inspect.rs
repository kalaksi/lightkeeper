use std::collections::HashMap;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
use lightkeeper_module::command_module;

#[command_module(
    "docker-inspect",
    "0.0.1",
    "Inspects a Docker container.
    Settings: none"
)]
pub struct Inspect;

impl Module for Inspect {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Inspect { }
    }
}

impl CommandModule for Inspect {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-containers"),
            parent_id: String::from("docker-containers"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("search"),
            display_text: String::from("Inspect"),
            action: UIAction::TextView,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, String> {
        let target_id = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if !string_validation::is_alphanumeric_with(target_id, &"-_") {
            panic!("Invalid container ID: {}", target_id)
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            let url = format!("http://localhost/containers/{}/json?all=true", target_id);
            command.arguments(vec!["curl", "--unix-socket", "/var/run/docker.sock", &url]);
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