
use std::collections::HashMap;
use std::str::FromStr;
use chrono::NaiveDateTime;
use serde_derive::Deserialize;
use serde_json;

use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::module::platform_info::Architecture;
use lightkeeper_module::command_module;


#[command_module(
    name="docker-image-remote-tags",
    version="0.0.1",
    description="Shows available remote tags for a Docker image. Supports Docker Registry API v2.",
)]
pub struct RemoteTags {
    page_size: u64,
    page_count: u64,
}

impl Module for RemoteTags {
    fn new(_settings: &HashMap<String, String>) -> Self {
        RemoteTags {
            // 100 is the maximum for the Docker Hub API.
            page_size: 100,
            page_count: 1,
        }
    }
}

impl CommandModule for RemoteTags {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("http-jwt", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-images"),
            parent_id: String::from("docker-images"),
            secondary_parent_id: String::from("docker-image-updates"),
            depends_on_no_tags: vec![String::from("Local")],
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("tag"),
            display_text: String::from("Show available tags"),
            action: UIAction::TextDialog,
            ..Default::default()
        }
    }

    fn get_connector_messages(&self, _host: Host, parameters: Vec<String>) -> Result<Vec<String>, LkError> {
        let _image_id = &parameters[0];
        let image_repo_tag = &parameters[1];

        if image_repo_tag.is_empty() {
            // Containers without a tag can not be used.
            return Err(LkError::other("Container has no tag and can not be used."))
        }

        let (image, _tag) = image_repo_tag.split_once(":").unwrap_or((image_repo_tag, ""));

        if image.contains(".") {
            // Looks like a domain name. Use Docker Registry API v2.
            let (domain, repository) = image.split_once("/").unwrap();
            Ok(vec![format!("https://{}/v2/{}/tags/list?n={}&last={}", domain, repository, self.page_size, 0)])
        }
        else {
            // Docker Hub.
            // Has it's own API with richer data.
            let (namespace, image) = image.split_once("/").unwrap_or(("library", image));
            let tags = (1..=self.page_count).map(|page| {
                format!("https://registry.hub.docker.com/v2/namespaces/{}/repositories/{}/tags?page_size={}&page={}", namespace, image, self.page_size, page)
            }).collect();
            Ok(tags)
        }
    }

    fn process_responses(&self, host: Host, responses: Vec<ResponseMessage>) -> Result<CommandResult, LkError> {
        let mut result_rows = Vec::new();
        let non_empty_responses = responses.iter().filter(|response| !response.message.is_empty()).collect::<Vec<_>>();

        for response in non_empty_responses.iter() {
            // Docker registry API v2.
            if let Ok(tags)  = serde_json::from_str(&response.message) {
                let mut tags: Tags = tags;
                tags.tags.sort();
                tags.tags.reverse();

                for tag in tags.tags.iter() {
                    result_rows.push(format!("- **{}**", tag));
                }
            }
            // Docker Hub API.
            else if let Ok(tags) = serde_json::from_str(&response.message) {
                let tags: DockerHubTags = tags;

                for tag_details in tags.results.iter() {
                    let images_for_arch = tag_details.images.iter()
                        .filter(|image_details| Architecture::from(&image_details.architecture) == host.platform.architecture)
                        .collect::<Vec<_>>();

                    if images_for_arch.len() > 1 {
                        result_rows.push(format!("- **{}**: *Error, too many images for arch {} found*", tag_details.name, host.platform.architecture));
                    }
                    else if images_for_arch.len() == 1 {
                        let image_details = match images_for_arch.first() {
                            Some(image_details) => image_details,
                            None => {
                                result_rows.push(format!("- **{}**: *Error parsing image details*", tag_details.name));
                                continue;
                            }
                        };

                        let last_pushed_string = if let Some(last_pushed_str) = &image_details.last_pushed {
                            let mut last_pushed = last_pushed_str.to_string();
                            for date_format in vec!["%Y-%m-%dT%H:%M:%S.%fZ", "%Y-%m-%dT%H:%M:%SZ"].iter() {
                                if let Ok(datetime) = NaiveDateTime::parse_from_str(last_pushed_str.as_str(), date_format) {
                                    let full_locale_string = std::env::var("LC_TIME").unwrap_or_else(|_| String::from("en_US.UTF-8"));
                                    let locale_string = full_locale_string.split(".").next().unwrap_or(&full_locale_string);
                                    let locale = chrono::Locale::from_str(&locale_string).unwrap_or_else(|_| chrono::Locale::en_US);
                                    last_pushed = datetime.and_utc().format_localized("last pushed %x %X UTC", locale).to_string();
                                    break;
                                }
                            }

                            last_pushed
                        }
                        else {
                            String::from("(Unknown)")
                        };

                        let image_size_in_mb = image_details.size / 1000 / 1000;
                        let image_size_string = format!("{} MB", image_size_in_mb);

                        result_rows.push(format!("- **{}**, {} ({})", tag_details.name, last_pushed_string, image_size_string));
                    }
                    else {
                        result_rows.push(format!("- **{}**: *No images for arch \"{}\"*", tag_details.name, host.platform.architecture));
                    }
                }
            }
            // Unknown.
            else {
                result_rows.push(format!("Error: unsupported registry API"));
            }
        }

        Ok(CommandResult::new_hidden(result_rows.join("\n")))
    }
}

#[derive(Deserialize)]
pub struct Tags {
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct DockerHubTags {
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