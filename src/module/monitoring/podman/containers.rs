/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;
use serde::Deserialize;
use serde_json;

use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::{ Host, enums::Criticality, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;
use crate::utils::ShellCommand;

#[monitoring_module(
    name="podman-containers",
    version="0.0.1",
    description="Provides information about Podman containers.",
    uses_sudo=true,
    settings={
      ignore_compose_managed => "Ignore containers that are managed by podman-compose. Default: true.",
      as_root => "Run podman with sudo as root. Default: true. If false, run as the SSH user (rootless)."
    }
)]
pub struct Containers {
    // Ignore containers that are managed by podman-compose.
    ignore_compose_managed: bool,
    as_root: bool,
}

impl Module for Containers {
    fn new(settings: &HashMap<String, String>) -> Self {
        Containers {
            ignore_compose_managed: settings.get("ignore_compose_managed").and_then(|value| Some(value == "true")).unwrap_or(true),
            as_root: settings.get("as_root").and_then(|value| Some(value == "true")).unwrap_or(true),
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
            category: String::from("podman-containers"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = self.as_root;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["podman", "ps", "-a", "--format", "json"]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.return_code != 0 {
            let result = DataPoint::value_with_level(String::from("Couldn't list Podman containers."), Criticality::Critical);
            return Ok(result);
        }

        let mut rows: Vec<PodmanPsJsonRow> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

        if self.ignore_compose_managed {
            rows.retain(|row| match &row.labels {
                None => true,
                Some(labels) => {
                    !labels.contains_key("com.docker.compose.config-hash")
                        && !labels.contains_key("io.podman.compose.config-hash")
                }
            });
        }

        let mut parent_data = DataPoint::empty();

        if !rows.is_empty() {
            if let Some(most_critical) = rows.iter().map(PodmanPsJsonRow::criticality).max() {
                parent_data.criticality = most_critical;
            }

            parent_data.multivalue = rows
                .iter()
                .map(|row| {
                    let mut point = DataPoint::value_with_level(row.state_display(), row.criticality());
                    point.label = row
                        .names
                        .iter()
                        .map(|name| cleanup_name(name))
                        .collect::<Vec<String>>()
                        .join(", ");
                    point.command_params = vec![cleanup_name(row.names.first().unwrap_or(&row.id))];
                    point
                })
                .collect();
        }

        Ok(parent_data)
    }
}

/// `podman ps --format json` row (Podman-specific; not Docker API-shaped).
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PodmanPsJsonRow {
    id: String,
    names: Vec<String>,
    #[serde(default)]
    labels: Option<HashMap<String, String>>,
    state: String,
    status: String,
    exited: bool,
    #[serde(default)]
    exit_code: i32,
}

impl PodmanPsJsonRow {
    fn state_display(&self) -> String {
        self.state.to_lowercase()
    }

    fn criticality(&self) -> Criticality {
        match self.state.to_lowercase().as_str() {
            "created" | "running" | "configured" => Criticality::Normal,
            "paused" => Criticality::Warning,
            "restarting" => Criticality::Error,
            "removing" => Criticality::Warning,
            "exited" | "stopped" => {
                if self.exited && self.exit_code == 0 {
                    Criticality::Normal
                }
                else if self.status.starts_with("Exited (0)") {
                    Criticality::Normal
                }
                else {
                    Criticality::Error
                }
            },
            "dead" => Criticality::Error,
            _ => Criticality::Warning,
        }
    }
}


pub fn cleanup_name(container_name: &str) -> String {
    let mut result = container_name.to_string();

    if container_name.starts_with('/') {
        result.remove(0);
    }

    result
}
