use std::{
    collections::HashMap,
    path::Path,
};

use crate::module::connection::ResponseMessage;
use crate::{ Host, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::monitoring::docker::containers::ContainerDetails;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("docker-compose", "0.0.1")]
pub struct Compose {
    use_sudo: bool,

    // TODO: these are unused atm
    pub compose_file_name: String,
    /// If you have one directory under which all the compose projects are, use this.
    pub main_dir: String, 
    /// If you have project directories all over the place, use this.
    pub project_directories: Vec<String>, 
}

impl Module for Compose {
    fn new(settings: &HashMap<String, String>) -> Self {
        Compose {
            use_sudo: settings.get("use_sudo").and_then(|value| Some(value == "true")).unwrap_or(true),

            compose_file_name: String::from("docker-compose.yml"),
            main_dir: settings.get("main_dir").unwrap_or(&String::new()).clone(),
            project_directories: settings.get("project_directories").unwrap_or(&String::new()).clone()
                                         .split(",").map(|value| value.to_string()).collect(),
        }
    }
}

impl MonitoringModule for Compose {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Compose"),
            category: String::from("docker-compose"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self) -> String {
        // Docker API is much better suited for this than using the docker-compose CLI.
        // More effective too.
        // TODO: Reuse command results between docker-compose and docker monitors (a global command cache?)
        // TODO: find down-status compose-projects with find-command?
        let command = String::from("curl --unix-socket /var/run/docker.sock http://localhost/containers/json?all=true");

        if self.use_sudo {
            format!("sudo {}", command)
        }
        else {
            command
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage) -> Result<DataPoint, String> {
        let mut containers: Vec<ContainerDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;
        containers.retain(|container| container.labels.contains_key("com.docker.compose.config-hash"));


        // There will be 2 levels of multivalues (services under projects).
        let mut projects_datapoint = DataPoint::empty();

        // Group containers by project name.
        let mut projects = HashMap::<String, Vec<DataPoint>>::new();

        for container in containers {
            let project = container.labels.get("com.docker.compose.project").unwrap().clone();

            if !projects.contains_key(&project) {
                projects.insert(project.clone(), Vec::new());
            }

            let service = container.labels.get("com.docker.compose.service").unwrap().clone();
            let compose_file = Path::new(container.labels.get("com.docker.compose.project.working_dir").unwrap())
                                    .join(&self.compose_file_name).to_string_lossy().to_string();

            let mut data_point = DataPoint::labeled_value_with_level(service.clone(), container.state.to_string(), container.state.to_criticality());
            data_point.command_params = vec![compose_file, service];

            projects.get_mut(&project).unwrap().push(data_point);
        }

        let mut projects_sorted = projects.keys().cloned().collect::<Vec<String>>();
        projects_sorted.sort();

        for project in projects_sorted.into_iter() {
            let mut datapoints = projects.remove_entry(&project).unwrap().1;
            datapoints.sort_by(|left, right| left.label.cmp(&right.label));

            let compose_file = datapoints.first().unwrap().command_params[0].clone();

            // Check just in case that all have the same compose-file.
            if datapoints.iter().any(|point| point.command_params[0] != compose_file) {
                panic!("Containers under same project can't have different compose-files");
            }

            let most_critical = datapoints.iter().max_by_key(|datapoint| datapoint.criticality).unwrap();
            let mut services_datapoint = DataPoint::labeled_value_with_level(project.clone(), most_critical.value.clone(), most_critical.criticality);
            services_datapoint.command_params = vec![compose_file];
            services_datapoint.multivalue = datapoints;

            projects_datapoint.multivalue.push(services_datapoint);
        }

        Ok(projects_datapoint)
    }
}