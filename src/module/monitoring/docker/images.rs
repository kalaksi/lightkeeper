/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;
use chrono::TimeZone;
use serde_derive::Deserialize;
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
    name="docker-images",
    version="0.0.1",
    description="Provides information about Docker images.",
    settings={
        age_warning_threshold => "Warning threshold in days. Default: 180",
        age_error_threshold => "Error threshold in days. Default: 365",
        age_critical_threshold => "Critical threshold in days. Default: 730",
        local_image_prefix => "Image name prefix indicating that image was built locally. Default: localhost",
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
            age_warning_threshold: settings.get("age_warning_threshold").unwrap_or(&String::from("180")).parse().unwrap(),
            age_error_threshold: settings.get("age_error_threshold").unwrap_or(&String::from("365")).parse().unwrap(),
            age_critical_threshold: settings.get("age_critical_threshold").unwrap_or(&String::from("730")).parse().unwrap(),
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
            display_text: String::from("Docker images"),
            category: String::from("docker-images"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "10") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            command.arguments(vec!["curl", "-s", "--unix-socket", "/var/run/docker.sock", "http://localhost/images/json"]);
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

        let images: Vec<ImageDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

        let mut root_point = DataPoint::empty();

        for image in images.iter() {
            let repo_tag = match &image.repo_tags {
                Some(repo_tags) => repo_tags.first().unwrap_or(&String::from("")).clone(),
                None => String::from(""),
            };

            let label = if repo_tag.is_empty() {
                image.id.clone()
            }
            else {
                repo_tag.clone()
            };

            let mut point = DataPoint::labeled_value(label, image.created.to_string());
            point.description = format!("Size: {} MB", image.size / 1024 / 1024);

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

            point.command_params = vec![image.id.clone(), repo_tag];
            root_point.multivalue.push(point);
        }

        Ok(root_point)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ImageDetails {
    id: String,
    created: i64,
    // labels: Option<HashMap<String, String>>,
    // parent_id: String,
    // repo_digests: Option<Vec<String>>,
    repo_tags: Option<Vec<String>>,
    // shared_size: i64,
    size: i64,
    // virtual_size: i64,
}
