
use std::collections::HashMap;
use std::str::FromStr;
use serde_derive::Deserialize;
use serde_json;
use chrono::{ NaiveDateTime, Utc };
use crate::Host;

use crate::module::{
    Module,
    Metadata,
    connection::ConnectionModule,
    monitoring::{MonitoringModule, MonitoringData},
    ModuleSpecification,
};

pub struct Docker {
    use_sudo: bool,
    excluded_containers: String,
}

impl Module for Docker {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from("docker"), String::from("0.0.1")),
            display_name: String::from("Docker"),
            description: String::from("Tested with API version 1.41"),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        Docker {
            use_sudo: settings.get("use_sudo").and_then(|value| Some(value == "true")).unwrap_or_else(|| false),
            excluded_containers: settings.get("excluded_containers").and_then(|value| Some(value.clone())).unwrap_or_else(|| String::from("")),
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Docker {
    fn get_connector_spec(&self) -> ModuleSpecification {
        ModuleSpecification::new(String::from("ssh"), String::from("0.0.1"))
    }

    fn refresh(&mut self, _host: &Host, connection: &mut Box<dyn ConnectionModule>) -> Result<MonitoringData, String> {
        // TODO: somehow connect directly to the unix socket instead of using curl?
        let mut command = String::from("curl --unix-socket /var/run/docker.sock http://localhost/containers/json?all=true");

        if self.use_sudo {
            command = format!("sudo {}", command);
        }

        let output = &connection.send_message(command.as_str())?;

        let mut containers: Vec<ContainerDetails> = serde_json::from_str(&output.as_str()).map_err(|e| e.to_string())?;
        containers.retain(|container| container.state != ContainerState::Running);
        let not_running_names: Vec<String> = containers.iter().map(|value| value.id.clone()).collect();
        
        if not_running_names.len() > 0 {
            return Ok(MonitoringData::new_with_level(
                not_running_names.join(", "),
                String::from("IDs"),
                crate::module::monitoring::Criticality::Error
            ));
        }
        Ok(MonitoringData::new_with_level(
            String::from(""),
            String::from("IDs"),
            crate::module::monitoring::Criticality::Normal
        ))
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ContainerDetails {
    id: String,
    state: ContainerState,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
enum ContainerState {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}