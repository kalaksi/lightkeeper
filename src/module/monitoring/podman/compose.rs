/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{
    collections::HashMap,
    path::Path,
};

use serde::Deserialize;
use serde_json;

use crate::enums::Criticality;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::{ Host, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;
use crate::utils::ShellCommand;

#[monitoring_module(
    name="podman-compose",
    version="0.0.1",
    description="Provides information about podman-compose projects.",
    uses_sudo=true,
    settings={
        compose_file_name => "Name of the podman-compose file. Default: docker-compose.yml",
        working_dir => "This is only needed with older podman-compose versions that don't include working_dir label on the container,
 so this can be used instead. Should be the parent directory of project directories. Multiple directory paths should be separated with a comma.",
        local_image_prefix => "Image name prefix indicating that image was built locally. Default: localhost"
    }
)]
pub struct Compose {
    pub compose_file_name: String,
    pub working_dir: String,
    pub local_image_prefix: String,
}

impl Module for Compose {
    fn new(settings: &HashMap<String, String>) -> Self {
        Compose {
            compose_file_name: settings.get("compose_file_name").unwrap_or(&String::from("docker-compose.yml")).clone(),
            working_dir: settings.get("working_dir").unwrap_or(&String::new()).clone(),
            local_image_prefix: settings.get("local_image_prefix").unwrap_or(&String::from("localhost")).clone(),
        }
    }
}

impl MonitoringModule for Compose {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Compose"),
            category: String::from("podman-compose"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = true;

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

        let mut rows: Vec<PodmanComposePsJsonRow> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;
        rows.retain(|row| {
            row.labels.as_ref().is_some_and(|labels| {
                labels.contains_key("com.docker.compose.config-hash")
                    || labels.contains_key("io.podman.compose.config-hash")
            })
        });

        // There will be 2 levels of multivalues (services under projects).
        let mut projects_datapoint = DataPoint::empty();

        // Group containers by project name.
        let mut projects = HashMap::<String, Vec<DataPoint>>::new();

        for row in rows {
            let labels = match &row.labels {
                Some(labels) => labels,
                None => {
                    log::info!("Container {} has no labels and therefore can't be used", row.id);
                    continue;
                }
            };

            // Try docker-compose labels first, then podman-compose labels
            let project = match labels.get("com.docker.compose.project") {
                Some(project) => project.clone(),
                None => {
                    match labels.get("io.podman.compose.project") {
                        Some(project) => project.clone(),
                        None => {
                            log::info!("Container {} has no compose project label and therefore can't be used", row.id);
                            continue;
                        }
                    }
                }
            };

            let project_datapoints = projects.entry(project.clone()).or_insert(Vec::new());

            let working_dir = match labels.get("com.docker.compose.project.working_dir") {
                Some(working_dir) => working_dir.clone(),
                None => {
                    match labels.get("io.podman.compose.project.working_dir") {
                        Some(working_dir) => working_dir.clone(),
                        None => {
                            log::warn!("Container {} has no compose project.working_dir label set.", row.id);
                            if !self.working_dir.is_empty() {
                                let working_dir = format!("{}/{}", self.working_dir, project);
                                log::warn!("User-defined working_dir \"{}\" is used instead. It isn't guaranteed that this is correct.", working_dir);
                                working_dir
                            }
                            else {
                                // Some earlier Compose versions don't include this label.
                                log::error!("working_dir setting has to be set in module settings with this version of Compose.");
                                continue;
                            }
                        }
                    }
                }
            };

            // Try docker-compose service label first, then podman-compose
            let service = match labels.get("com.docker.compose.service") {
                Some(service) => service.clone(),
                None => {
                    match labels.get("io.podman.compose.service") {
                        Some(service) => service.clone(),
                        None => {
                            return Err(String::from("Container has no compose service label"));
                        }
                    }
                }
            };
            let compose_file = Path::new(&working_dir)
                                    .join(&self.compose_file_name).to_string_lossy().to_string();

            let mut data_point =
                DataPoint::labeled_value_with_level(service.clone(), row.status.clone(), row.criticality());

            if row.image.starts_with(&self.local_image_prefix) || row.image.starts_with("sha256:") {
                data_point.tags.push(String::from("Local"));
            }

            data_point.description = row.image.clone();
            data_point.command_params = vec![compose_file, project.clone(), service];

            project_datapoints.push(data_point);
        }

        let mut projects_sorted = projects.keys().cloned().collect::<Vec<String>>();
        projects_sorted.sort();

        for project in projects_sorted {
            let mut data_points = projects.remove_entry(&project).ok_or(LkError::unexpected())?.1;
            data_points.sort_by(|left, right| left.label.cmp(&right.label));

            let compose_file = match data_points.first() {
                Some(first) => first.command_params[0].clone(),
                None => {
                    log::error!("No compose-file found for project {}", project);
                    continue;
                }
            };

            // Check just in case that all have the same compose-file.
            if data_points.iter().any(|point| point.command_params[0] != compose_file) {
                log::error!("Containers under same project can't have different compose-files");
                continue;
            }

            if let Some(most_critical) = data_points.iter().max_by_key(|datapoint| datapoint.criticality) {
                projects_datapoint.criticality = std::cmp::max(projects_datapoint.criticality, most_critical.criticality);

                let mut services_datapoint = DataPoint::labeled_value_with_level(project.clone(), most_critical.value.clone(), most_critical.criticality);
                services_datapoint.command_params = vec![compose_file, project.clone()];
                services_datapoint.multivalue = data_points;
                projects_datapoint.multivalue.push(services_datapoint);
            }
        }

        Ok(projects_datapoint)
    }
}

/// `podman ps --format json` row for compose monitoring (Podman-specific).
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PodmanComposePsJsonRow {
    id: String,
    image: String,
    status: String,
    state: String,
    exited: bool,
    #[serde(default)]
    exit_code: i32,
    #[serde(default)]
    labels: Option<HashMap<String, String>>,
}

impl PodmanComposePsJsonRow {
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
