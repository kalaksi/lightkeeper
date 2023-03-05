
use std::collections::HashMap;
use std::fmt;
use serde_derive::Deserialize;
use serde_json;

use crate::module::connection::ResponseMessage;
use crate::{ Host, enums::Criticality, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("docker-containers", "0.0.1")]
pub struct Containers {
    // Ignore containers that are managed by docker-compose.
    ignore_compose_managed: bool,
}

impl Module for Containers {
    fn new(settings: &HashMap<String, String>) -> Self {
        Containers {
            ignore_compose_managed: settings.get("ignore_compose_managed").and_then(|value| Some(value == "true")).unwrap_or(true),
        }
    }
}

impl MonitoringModule for Containers {
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

    fn get_connector_message(&self, host: Host) -> String {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            // TODO: somehow connect directly to the unix socket instead of using curl?
            let command = String::from("curl --unix-socket /var/run/docker.sock http://localhost/containers/json?all=true");

            if host.settings.contains(&crate::host::HostSetting::UseSudo) {
                format!("sudo {}", command)
            }
            else {
                command
            }
        }
        else {
            String::new()
        }
    }

    fn process_response(&self, host: Host, response: ResponseMessage) -> Result<DataPoint, String> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            let mut containers: Vec<ContainerDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

            if self.ignore_compose_managed {
                containers.retain(|container| !container.labels.contains_key("com.docker.compose.config-hash"));
            }

            let mut parent_data = DataPoint::empty();

            if !containers.is_empty() {
                let most_critical_container = containers.iter().max_by_key(|container| container.state.to_criticality()).unwrap();
                parent_data.criticality = most_critical_container.state.to_criticality();

                parent_data.multivalue = containers.iter().map(|container| {
                    let mut point = DataPoint::value_with_level(container.state.to_string(), container.state.to_criticality());
                    // Names may still contain a leading slash that can cause issues with docker commands.
                    point.label = container.names.iter().map(|name| cleanup_name(name)).collect::<Vec<String>>().join(", ");
                    point.command_params = vec![cleanup_name(&container.names.first().unwrap_or(&container.id))];
                    point
                }).collect();
            }

            Ok(parent_data)
        }
        else {
            self.error_unsupported()
        }
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