use std::collections::HashMap;
use crate::frontend;
use crate::module::{
    Module,
    command::CommandModule,
    command::Command,
    command::SubCommand,
    Metadata,
    ModuleSpecification,
};

use super::CommandResult;

#[derive(Clone)]
pub struct Docker;

impl Module for Docker {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Docker { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Docker {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker"),
            parent_id: String::from("docker"),
            ..Default::default()
        }
    }

    fn get_subcommands(&self) -> Vec<SubCommand> {
        vec![
            SubCommand::new_with_icon("restart", "refresh"),
            SubCommand::new_with_icon("inspect", "search"),
            SubCommand::new_with_icon("shell", "terminal"),
        ]
    }

    fn get_connector_request(&self, subcommand: String, target_id: String) -> String {
        if target_id.is_empty() {
            panic!("target_id is mandatory and should contain a container ID");
        }

        match subcommand.as_str() {
            "restart" => format!("sudo curl --unix-socket /var/run/docker.sock -X POST http://localhost/containers/{}/restart", target_id),
            "inspect" => format!("sudo curl --unix-socket /var/run/docker.sock http://localhost/containers/{}/json?all=true", target_id),
            "shell" => String::from("TODO"),
            _ => panic!("Unknown subcommand: {}", subcommand),
        }
    }

    fn process_response(&self, response: &String) -> Result<CommandResult, String> {
        log::debug!("Got response: {}", response);
        Ok(CommandResult::new(String::from(response)))
    }
}