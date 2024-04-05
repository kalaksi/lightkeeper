
use std::collections::HashMap;
use chrono::NaiveDateTime;
use serde_derive::Deserialize;
use serde_json;
use chrono::Utc;

use crate::module::connection::ResponseMessage;
use crate::enums::Criticality;
use crate::Host;
use crate::frontend;
use crate::module::platform_info::Architecture;
use lightkeeper_module::monitoring_extension_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_extension_module(
    name="docker-image-updates",
    version="0.0.1",
    parent_module_name="docker-images",
    parent_module_version="0.0.1",
    description= "Checks if there are updates available for Docker image tags.",
)]
pub struct ImageUpdates {
}

impl Module for ImageUpdates {
    fn new(_settings: &HashMap<String, String>) -> Self {
        ImageUpdates {
        }
    }
}

impl MonitoringModule for ImageUpdates {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("http", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Docker image updates"),
            category: String::from("docker-images"),
            override_summary_monitor_id: String::from("docker-images"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_messages(&self, _host: Host, parent_result: DataPoint) -> Result<Vec<String>, String> {
        if parent_result.is_empty() {
            return Ok(Vec::new());
        }

        let result = parent_result.multivalue.iter().map(|data_point| {
            let image_repo_tag = data_point.command_params.get(1).unwrap();

            if data_point.tags.contains(&String::from("Local")) ||
               image_repo_tag.is_empty() {
                // Containers without a tag and local images can not be used.
                String::new()
            }
            else {
                let (image, tag) = image_repo_tag.split_once(":").unwrap_or(("", ""));

                // Probably contains non-dockerhub registry address in image name.
                if image.matches("/").count() > 1 {
                    return String::new();
                }

                let (namespace, image) = image.split_once("/").unwrap_or(("library", image));

                // TODO: support other registries too.
                format!("https://registry.hub.docker.com/v2/repositories/{}/{}/tags/{}", namespace, image, tag)
            }
        }).collect();

        Ok(result)
    }

    fn process_responses(&self, host: Host, responses: Vec<ResponseMessage>, parent_result: DataPoint) -> Result<DataPoint, String> {
        if responses.is_empty() {
            return Ok(DataPoint::empty());
        }

        let mut new_result = parent_result;

        if responses.len() != new_result.multivalue.len() {
            return Err(String::from("Invalid amount of responses"));
        }

        new_result.multivalue = new_result.multivalue.into_iter().enumerate().map(|(index, old_point)| {
            let mut new_point = old_point.clone();
            let image_repo_tag = old_point.command_params.get(1).unwrap();

            // Responses are in the same order as the connector messages in get_connector_messages.
            let response = responses.get(index).unwrap();

            new_point.is_from_cache = response.is_from_cache && old_point.is_from_cache;

            if response.is_error() {
                new_point = DataPoint::empty_and_critical();
            }
            else if response.is_empty() {
                new_point.description = old_point.value;
                if new_point.tags.contains(&String::from("Local")) {
                    new_point.criticality = Criticality::Normal;
                    new_point.value = String::from("Up-to-date");
                }
                else if image_repo_tag.is_empty() {
                    new_point.criticality = Criticality::Warning;
                    new_point.value = String::from("N/A");
                }
            }
            else if let Ok(tag_details) = serde_json::from_str::<TagDetails>(response.message.as_str()) {
                let mut images_for_arch = tag_details.images.iter()
                    .filter(|image_details| Architecture::from(&image_details.architecture) == host.platform.architecture)
                    .collect::<Vec<_>>();

                if images_for_arch.len() >= 1 {
                    // last_pushed is a string but because of the format sorting also works as string.
                    images_for_arch.sort_by(|a, b| a.last_pushed.cmp(&b.last_pushed));
                    let image_details = images_for_arch.last().unwrap();
                    let last_pushed = NaiveDateTime::parse_from_str(image_details.last_pushed.as_str(), "%Y-%m-%dT%H:%M:%S.%fZ").unwrap().and_utc();
                    let local_image_age = old_point.value.split_once(" ").unwrap().0.parse::<i64>().unwrap();
                    // When local image was pulled.
                    let last_pulled = Utc::now() - chrono::Duration::days(local_image_age);

                    if last_pushed > last_pulled {
                        // Retain old criticality if it was higher.
                        new_point.criticality = new_point.criticality.max(Criticality::Warning);
                        new_point.value = String::from("Outdated");
                    } else {
                        new_point.criticality = Criticality::Normal;
                        new_point.value = String::from("Up-to-date");
                    }

                    new_point.description = old_point.value;
                }
            }
            else {
                new_point = DataPoint::empty_and_critical();
            }

            new_point
        }).collect::<Vec<DataPoint>>();

        Ok(new_result)
    }
}

#[derive(Deserialize)]
pub struct TagDetails {
    pub name: String,
    pub images: Vec<ImageDetails>,
}

#[derive(Deserialize)]
pub struct ImageDetails {
    pub architecture: String,
    pub os: String,
    pub variant: Option<String>,
    pub size: i64,
    pub status: String,
    pub last_pushed: String,
}
