use std::{
    collections::HashMap,
    path::Path,
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

        // There will be 2 levels of multivalues.
        let mut parent_point = DataPoint::empty();
        let most_critical_container = containers.iter().max_by_key(|container| container.state.to_criticality()).unwrap();
        parent_point.criticality = most_critical_container.state.to_criticality();

        // Group containers by project name.
        let mut projects = HashMap::<String, Vec<DataPoint>>::new();

        for container in containers {
            let project = container.labels.get("com.docker.compose.project").unwrap().clone();

            if let Some(project_datapoints) = projects.get_mut(&project) {
                let service = container.labels.get("com.docker.compose.service").unwrap().clone();
                let container_number = container.labels.get("com.docker.compose.container-number").unwrap().clone();
                let container_name = [project.clone(), service.clone(), container_number].join("_");

                let mut data_point = DataPoint::labeled_value_with_level(service, container.state.to_string(), container.state.to_criticality());
                data_point.source_id = container_name;
                project_datapoints.push(data_point);
            }
            else {
                projects.insert(project, Vec::new());
            }
        }

        for (project, datapoints) in projects {
            let mut second_parent_point = DataPoint::empty();
            second_parent_point.label = project.clone();
            second_parent_point.source_id = project;
            second_parent_point.multivalue = datapoints;

            parent_point.multivalue.push(second_parent_point);
        }

        Ok(parent_point)
    }
}


#[derive(Clone)]
pub struct ComposeConfig {
    pub compose_file_name: String,
    /// If you have one directory under which all the compose projects are, use this.
    pub main_dir: String, 
    /// If you have project directories all over the place, use this.
    pub project_directories: Vec<String>, 
}

// TODO: some validations?
impl ComposeConfig {
    pub fn new(settings: &HashMap<String, String>) -> Self {
        ComposeConfig {
            compose_file_name: String::from("docker-compose.yml"),
            main_dir: settings.get("main_dir").unwrap_or(&String::new()).clone(),
            project_directories: settings.get("project_directories").unwrap_or(&String::new()).clone()
                                         .split(",").map(|value| value.to_string()).collect(),
        }
    }


    pub fn get_project_dir(&self, project_name: &String) -> String {
        if !self.main_dir.is_empty() {
            return Path::new(&self.main_dir).join(project_name.clone()).to_string_lossy().to_string();
        }
        else {
            for dir in self.project_directories.iter() {
                // The last directory component should match the project name.
                let last_dir_component = Path::new(dir).components().last().unwrap().as_os_str().to_string_lossy();
                if *project_name == last_dir_component {
                    return dir.clone();
                }
            }
        }
        panic!()
    }

    pub fn get_project_compose_file(&self, project_name: String) -> String {
        let project_dir = self.get_project_dir(&project_name);
        Path::new(&project_dir).join(self.compose_file_name.clone()).to_string_lossy().to_string()
    }

}