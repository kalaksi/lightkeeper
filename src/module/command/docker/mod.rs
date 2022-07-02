use std::collections::HashMap;
use crate::module::{
    Module,
    command::CommandModule,
    Metadata,
    ModuleSpecification,
};

use super::CommandResult;

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
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_subcommands(&self) -> Option<Vec<String>> {
        Some(vec![
            String::from("ps"),
            String::from("images")
        ])
    }

    fn get_connector_request(&self, subcommand: Option<String>) -> String {
        /* 
        match subcommand.unwrap().as_str() {
            "ps" => String::from("docker ps"),
            "images" => String::from("docker images"),
        }*/
        String::from("sudo curl --unix-socket /var/run/docker.sock http://localhost/containers/json?all=true")
    }

    fn process_response(&self, response: &String) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.clone()))
    }
}