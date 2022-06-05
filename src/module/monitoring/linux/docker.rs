
use std::collections::HashMap;
use std::fmt;
use serde_derive::Deserialize;
use serde_json;
use crate::Host;

use crate::module::{ Module, Metadata, connection::ConnectionModule, ModuleSpecification };
use crate::module::monitoring::{ MonitoringModule, Criticality, DisplayStyle, DisplayOptions, DataPoint };

pub struct Docker {
    use_sudo: bool,
    excluded_containers: String,
}

impl Module for Docker {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from("docker"), String::from("0.0.1")),
            description: String::from("Tested with API version 1.41"),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        Docker {
            use_sudo: settings.get("use_sudo").and_then(|value| Some(value == "true")).unwrap_or(false),
            excluded_containers: settings.get("excluded_containers").unwrap_or(&String::from("")).clone(),
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

    fn get_display_options(&self) -> DisplayOptions {
        DisplayOptions {
            display_style: DisplayStyle::CriticalityLevel,
            display_name: String::from("Docker containers"),
            use_multivalue: true,
            unit: String::from(""),
        }
    }

    fn refresh(&mut self, _host: &Host, connection: &mut Box<dyn ConnectionModule>) -> Result<DataPoint, String> {
        // TODO: somehow connect directly to the unix socket instead of using curl?
        let mut command = String::from("curl --unix-socket /var/run/docker.sock http://localhost/containers/json?all=true");

        if self.use_sudo {
            command = format!("sudo {}", command);
        }

        let output = &connection.send_message(command.as_str())?;

        let containers: Vec<ContainerDetails> = serde_json::from_str(&output.as_str()).map_err(|e| e.to_string())?;

        let mut parent_data = DataPoint::empty();
        let most_critical_container = containers.iter().max_by_key(|container| container.state.to_criticality()).unwrap();
        parent_data.criticality = most_critical_container.state.to_criticality();
        parent_data.multivalue = containers.iter().map(|container| {
            DataPoint::new_with_level(container.state.to_string(), container.state.to_criticality())
        }).collect();

        Ok(parent_data)
    }

}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ContainerDetails {
    id: String,
    names: Vec<String>,
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

impl ContainerState {
    fn to_criticality(&self) -> Criticality {
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