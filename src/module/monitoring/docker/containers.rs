/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;
use std::fmt;
use serde_derive::Deserialize;
use serde_json;

use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::{ Host, enums::Criticality, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;
use crate::utils::ShellCommand;

#[monitoring_module(
    name="docker-containers",
    version="0.0.1",
    description="Provides information about Docker containers.",
    uses_sudo=true,
    settings={
      ignore_compose_managed => "Ignore containers that are managed by docker-compose. Default: true."
    }
)]
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
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
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

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = true;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            // TODO: somehow connect directly to the unix socket instead of using curl?
            command.arguments(vec!["curl", "-s", "--unix-socket", "/var/run/docker.sock", "http://localhost/containers/json?all=true"]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.return_code == 7 {
            // Coudldn't connect. Daemon is probably not available.
            let result = DataPoint::value_with_level(String::from("Couldn't connect to Docker daemon."), Criticality::Critical);
            return Ok(result);
        }

        let mut containers: Vec<ContainerDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

        if self.ignore_compose_managed {
            containers.retain(|container| !container.labels.contains_key("com.docker.compose.config-hash"));
        }

        let mut parent_data = DataPoint::empty();

        if !containers.is_empty() {
            if let Some(most_critical_container) = containers.iter().max_by_key(|container| container.get_criticality()) {
                parent_data.criticality = most_critical_container.get_criticality();
            }

            parent_data.multivalue = containers.iter().map(|container| {
                let mut point = DataPoint::value_with_level(container.state.to_string(), container.get_criticality());
                // Names may still contain a leading slash that can cause issues with docker commands.
                point.label = container.names.iter().map(|name| cleanup_name(name)).collect::<Vec<String>>().join(", ");
                point.command_params = vec![cleanup_name(container.names.first().unwrap_or(&container.id))];
                point
            }).collect();
        }

        Ok(parent_data)
    }
}


pub fn cleanup_name(container_name: &str) -> String {
    let mut result = container_name.to_string();

    if container_name.starts_with('/') {
        result.remove(0);
    }

    result
}


#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerDetails {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub state: ContainerState,
    pub status: String,
    pub ports: Vec<ContainerPort>,
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct ContainerPort {
    pub ip: Option<String>,
    pub private_port: Option<u16>,
    pub public_port: Option<u16>,
    pub type_: Option<String>,
}

#[derive(Deserialize, PartialEq)]
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

impl ContainerDetails {
    pub fn get_criticality(&self) -> Criticality {
        match self.state {
            ContainerState::Created => Criticality::Normal,
            ContainerState::Running => Criticality::Normal,
            ContainerState::Paused => Criticality::Warning,
            ContainerState::Restarting => Criticality::Error,
            ContainerState::Removing => Criticality::Warning,
            ContainerState::Exited => {
                if self.status.starts_with("Exited (0)") {
                    Criticality::Normal
                }
                else {
                    Criticality::Error
                }
            },
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