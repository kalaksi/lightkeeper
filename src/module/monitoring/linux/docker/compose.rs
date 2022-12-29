use std::{
    collections::HashMap,
};

use crate::module::connection::ResponseMessage;
use crate::{ Host, frontend };
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::DataPoint,
    monitoring::docker::containers::ContainerDetails,
};


#[derive(Clone)]
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
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-compose", "0.0.1"),
            // TODO: check compose version and enforce compatibility
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        Compose {
            use_sudo: settings.get("use_sudo").and_then(|value| Some(value == "true")).unwrap_or(true),

            compose_file_name: String::from("docker-compose.yml"),
            main_dir: settings.get("main_dir").unwrap_or(&String::new()).clone(),
            project_directories: settings.get("project_directories").unwrap_or(&String::new()).clone()
                                         .split(",").map(|value| value.to_string()).collect(),
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Compose {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

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

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        let mut containers: Vec<ContainerDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;
        containers.retain(|container| container.labels.contains_key("com.docker.compose.config-hash"));


        // There will be 2 levels of multivalues (services under projects).
        let mut parent_point = DataPoint::empty();
        let most_critical_container = containers.iter().max_by_key(|container| container.state.to_criticality()).unwrap();
        parent_point.criticality = most_critical_container.state.to_criticality();

        // Group containers by project name.
        let mut projects = HashMap::<String, Vec<DataPoint>>::new();

        for container in containers {
            let project = container.labels.get("com.docker.compose.project").unwrap().clone();

            if !projects.contains_key(&project) {
                projects.insert(project.clone(), Vec::new());
            }

            let service = container.labels.get("com.docker.compose.service").unwrap().clone();
            let container_number = container.labels.get("com.docker.compose.container-number").unwrap().clone();
            let container_name = [project.clone(), service.clone(), container_number].join("_");
            let compose_file = container.labels.get("com.docker.compose.project.working_dir").unwrap().clone();

            let mut data_point = DataPoint::labeled_value_with_level(service, container.state.to_string(), container.state.to_criticality());
            data_point.command_params = vec![container_name, compose_file];

            projects.get_mut(&project).unwrap().push(data_point);
        }

        for (project, datapoints) in projects {
            let compose_file = datapoints.first().unwrap().command_params[1].clone();

            // Check just in case that all have the same compose-file.
            if datapoints.iter().any(|point| point.command_params[1] != compose_file) {
                panic!("Containers under same project can't have different compose-files");
            }

            let mut second_parent_point = DataPoint::none();
            second_parent_point.label = project.clone();
            second_parent_point.command_params = vec![project, compose_file];
            second_parent_point.multivalue = datapoints;

            parent_point.multivalue.push(second_parent_point);
        }

        Ok(parent_point)
    }
}