
use std::collections::HashMap;
use chrono::TimeZone;
use serde_derive::Deserialize;
use serde_json;

use crate::module::connection::ResponseMessage;
use crate::{ Host, frontend };
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::DataPoint,
};

use chrono::Utc;


#[derive(Clone)]
pub struct Images {
    use_sudo: bool,
}

impl Module for Images {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-images", "0.0.1"),
            description: String::from("Tested with Docker API version 1.41"),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        Images {
            use_sudo: settings.get("use_sudo").and_then(|value| Some(value == "true")).unwrap_or(false),
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Images {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
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

    fn get_connector_message(&self) -> String {
        // TODO: somehow connect directly to the unix socket instead of using curl?
        let command = String::from("curl --unix-socket /var/run/docker.sock http://localhost/images/json");

        if self.use_sudo {
            format!("sudo {}", command)
        }
        else {
            command
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        let images: Vec<ImageDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

        let mut parent_data = DataPoint::empty();

        parent_data.multivalue = images.iter().map(|image| {
            let label = match &image.repo_tags {
                Some(repo_tags) => repo_tags.first().unwrap().clone(),
                None => image.id.clone(),
            };
            let mut point = DataPoint::labeled_value(label, image.created.to_string());
            point.command_params = vec![image.id.clone()];

            // TODO: how to make sure timezone is accounted for correctly?
            let creation_time = Utc.timestamp(point.value.parse::<i64>().unwrap(), 0);
            let duration = Utc::now().signed_duration_since(creation_time);
            point.value = format!("{} days", duration.num_days());

            point
        }).collect();

        Ok(parent_data)
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
    // size: i64,
    // virtual_size: i64,
}
