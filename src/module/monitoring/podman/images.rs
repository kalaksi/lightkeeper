/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;

use chrono::TimeZone;
use serde::Deserialize;
use serde_json;
use chrono::Utc;

use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::enums::Criticality;
use crate::{ Host, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;
use crate::utils::ShellCommand;


#[monitoring_module(
    name="podman-images",
    version="0.0.1",
    description="Provides information about Podman images.",
    uses_sudo=true,
    settings={
        age_warning_threshold => "Warning threshold in days. Default: 180",
        age_error_threshold => "Error threshold in days. Default: 365",
        age_critical_threshold => "Critical threshold in days. Default: 730",
        local_image_prefix => "Image name prefix indicating that image was built locally. Default: localhost"
    }
)]
pub struct Images {
    age_warning_threshold: i64,
    age_error_threshold: i64,
    age_critical_threshold: i64,
    local_image_prefix: String,
}

impl Module for Images {
    fn new(settings: &HashMap<String, String>) -> Self {
        Images {
            age_warning_threshold: settings.get("age_warning_threshold").and_then(|value| value.parse().ok()).unwrap_or(180),
            age_error_threshold: settings.get("age_error_threshold").and_then(|value| value.parse().ok()).unwrap_or(365),
            age_critical_threshold: settings.get("age_critical_threshold").and_then(|value| value.parse().ok()).unwrap_or(730),
            local_image_prefix: settings.get("local_image_prefix").unwrap_or(&String::from("localhost")).clone(),
        }
    }
}

impl MonitoringModule for Images {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Podman images"),
            category: String::from("podman-images"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = true;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["podman", "images", "--format", "json"]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.return_code != 0 {
            let result = DataPoint::value_with_level(String::from("Couldn't list Podman images."), Criticality::Critical);
            return Ok(result);
        }

        let rows: Vec<PodmanImagesJsonRow> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

        let mut root_point = DataPoint::empty();

        for row in rows.iter() {
            let repo_tag = row.primary_reference();

            let label = if repo_tag.is_empty() {
                row.id.clone()
            }
            else {
                repo_tag.clone()
            };

            let mut point = DataPoint::labeled_value(label, row.created.to_string());
            point.description = format!("Size: {} MB", row.size / 1024 / 1024);

            // TODO: make sure timezone is accounted for correctly?
            if let Ok(parsed_value) = point.value.parse::<i64>() {
                let creation_time = Utc.timestamp_opt(parsed_value, 0).unwrap();
                let duration_days = Utc::now().signed_duration_since(creation_time).num_days();

                if duration_days >= self.age_critical_threshold {
                    point.criticality = Criticality::Critical;
                }
                else if duration_days >= self.age_error_threshold {
                    point.criticality = Criticality::Error;
                }
                else if duration_days >= self.age_warning_threshold {
                    point.criticality = Criticality::Warning;
                }

                point.value = format!("{} days old", duration_days);
            }
            else {
                point.value = String::from("Parse error");
                point.criticality = Criticality::Error;
            }

            if repo_tag.starts_with(&self.local_image_prefix) {
                point.tags.push(String::from("Local"));
            }

            point.command_params = vec![row.id.clone(), repo_tag];
            root_point.multivalue.push(point);
        }

        root_point.update_criticality_from_children();
        Ok(root_point)
    }
}

/// `podman images --format json` row (Podman-specific; not Docker API-shaped).
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PodmanImagesJsonRow {
    id: String,
    created: i64,
    #[serde(default)]
    repo_tags: Option<Vec<String>>,
    #[serde(default)]
    names: Option<Vec<String>>,
    size: i64,
}

impl PodmanImagesJsonRow {
    fn primary_reference(&self) -> String {
        if let Some(tags) = &self.repo_tags {
            if let Some(first) = tags.first() {
                if !first.is_empty() {
                    return first.clone();
                }
            }
        }
        if let Some(names) = &self.names {
            if let Some(first) = names.first() {
                if !first.is_empty() {
                    return first.clone();
                }
            }
        }
        String::new()
    }
}
