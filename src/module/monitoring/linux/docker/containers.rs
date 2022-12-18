
use std::collections::HashMap;
use std::fmt;
use serde_derive::Deserialize;
use serde_json;

use crate::module::connection::ResponseMessage;
use crate::{ Host, utils::enums::Criticality, frontend };
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::DataPoint,
};


#[derive(Clone)]
pub struct Containers {
    use_sudo: bool,
    // Ignore containers that are managed by docker-compose.
    ignore_compose_managed: bool,
}

impl Module for Containers {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-containers", "0.0.1"),
            description: String::from("Tested with Docker API version 1.41"),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        Containers {
            use_sudo: settings.get("use_sudo").and_then(|value| Some(value == "true")).unwrap_or(false),
            ignore_compose_managed: true,
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Containers {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Containers"),
            category: String::from("docker-containers"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self) -> String {
        // TODO: somehow connect directly to the unix socket instead of using curl?
        let command = String::from("curl --unix-socket /var/run/docker.sock http://localhost/containers/json?all=true");

        if self.use_sudo {
            format!("sudo {}", command)
        }
        else {
            command
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        let mut containers: Vec<ContainerDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

        let mut parent_data = DataPoint::empty();
        let most_critical_container = containers.iter().max_by_key(|container| container.state.to_criticality()).unwrap();
        parent_data.criticality = most_critical_container.state.to_criticality();

        if self.ignore_compose_managed {
            containers.retain(|container| !container.labels.contains_key("com.docker.compose.config-hash"));
        }

        parent_data.multivalue = containers.iter().map(|container| {
            let mut point = DataPoint::value_with_level(container.state.to_string(), container.state.to_criticality());
            // Names may still contain a leading slash that can cause issues with docker commands.
            point.label = container.names.iter().map(|name| cleanup_name(name)).collect::<Vec<String>>().join(", ");
            point.source_id = cleanup_name(&container.names.first().unwrap_or(&container.id));
            point
        }).collect();

        Ok(parent_data)
    }

}


pub fn cleanup_name(container_name: &String) -> String {
    let mut result = container_name.clone();

    if container_name.chars().next().unwrap() == '/' {
        result.remove(0);
    }

    result
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerDetails {
    pub id: String,
    pub names: Vec<String>,
    pub state: ContainerState,
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}

impl ContainerState {
    pub fn to_criticality(&self) -> Criticality {
        match self {
            ContainerState::Created => Criticality::Normal,
            ContainerState::Running => Criticality::Normal,
            ContainerState::Paused => Criticality::Warning,
            ContainerState::Restarting => Criticality::Error,
            ContainerState::Removing => Criticality::Warning,
            ContainerState::Exited => Criticality::Error,
            ContainerState::Dead => Criticality::Error,
        }
    }
}

impl fmt::Display for ContainerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContainerState::Created => write!(f, "created"),
            ContainerState::Running => write!(f, "running"),
            ContainerState::Paused => write!(f, "paused"),
            ContainerState::Restarting => write!(f, "restarting"),
            ContainerState::Removing => write!(f, "removing"),
            ContainerState::Exited => write!(f, "exited"),
            ContainerState::Dead => write!(f, "dead"),
        }
    }
}