
use std::collections::HashMap;
use chrono::TimeZone;
use chrono::Utc;
use serde_derive::Deserialize;
use serde_json;

use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::module::platform_info::Architecture;
use lightkeeper_module::command_module;


#[command_module("docker-image-remote-tags", "0.0.1")]
pub struct RemoteTags {
    page_size: u64,
    page_count: u64,
}

impl Module for RemoteTags {
    fn new(_settings: &HashMap<String, String>) -> Self {
        RemoteTags {
            // 100 is the maximum for the Docker Hub API.
            page_size: 100,
            page_count: 2,
        }
    }
}

impl CommandModule for RemoteTags {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("http", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-images"),
            parent_id: String::from("docker-image-updates"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("tag"),
            display_text: String::from("Show available tags"),
            action: UIAction::TextDialog,
            ..Default::default()
        }
    }

    fn get_connector_messages(&self, _host: Host, parameters: Vec<String>) -> Result<Vec<String>, String> {
        let _image_id = &parameters[0];
        let image_repo_tag = &parameters[1];

        if image_repo_tag.is_empty() {
            // Containers without a tag can not be used.
            Err(String::from("Container has no tag and can not be used."))
        }
        else {
            let (image, _tag) = image_repo_tag.split_once(":").unwrap_or(("", ""));
            let (namespace, image) = image.split_once("/").unwrap_or(("library", image));

            // TODO: support other registries too.
            let tags = (1..=self.page_count).map(|page| {
                format!("https://registry.hub.docker.com/v2/namespaces/{}/repositories/{}/tags?page_size={}&page={}", namespace, image, self.page_size, page)
            }).collect();
            Ok(tags)
        }
    }

    fn process_responses(&self, host: Host, responses: Vec<ResponseMessage>) -> Result<CommandResult, String> {
        let mut result_rows = Vec::new();
        let non_empty_responses = responses.iter().filter(|response| !response.message.is_empty()).collect::<Vec<_>>();

        for response in non_empty_responses.iter() {
            let tags: Tags = serde_json::from_str(&response.message).unwrap();
            for tag_details in tags.results.iter() {
                let images_for_arch = tag_details.images.iter()
                    .filter(|image_details| Architecture::from(&image_details.architecture) == host.platform.architecture)
                    .collect::<Vec<_>>();

                if images_for_arch.len() > 1 {
                    result_rows.push(format!("**{}**: (Error, too many images for arch {} found)", tag_details.name, host.platform.architecture));
                }
                else if images_for_arch.len() == 1 {
                    let image_details = images_for_arch.first().unwrap();
                    let last_pushed_formatted = if let Some(last_pushed_str) = &image_details.last_pushed {
                        let datetime = Utc.datetime_from_str(last_pushed_str.as_str(), "%Y-%m-%dT%H:%M:%S.%fZ").unwrap();
                        // TODO: format according to locale.
                        datetime.format("%d.%m.%Y %H:%M:%S").to_string()
                    }
                    else {
                        String::from("(Unknown)")
                    };

                    let image_size_in_mb = image_details.size / 1024 / 1024;
                    result_rows.push(format!("- **{}**, last pushed {} UTC ({} MB)", tag_details.name, last_pushed_formatted, image_size_in_mb));
                }
            }
        }

        Ok(CommandResult::new(result_rows.join("\n")))
    }
}

#[derive(Deserialize)]
pub struct Tags {
    pub count: u64,
    pub previous: Option<String>,
    pub next: Option<String>,
    pub results: Vec<TagDetails>,
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
    pub last_pushed: Option<String>,
}