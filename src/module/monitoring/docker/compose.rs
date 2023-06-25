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
use crate::utils::ShellCommand;

#[monitoring_module("docker-compose", "0.0.1")]
pub struct Compose {
    pub compose_file_name: String,
    /// Earlier docker-compose versions don't include working_dir label so this can be used instead.
    /// Currently, a single directory is supported.
    pub working_dir: String, 
}

impl Module for Compose {
    fn new(settings: &HashMap<String, String>) -> Self {
        Compose {
            compose_file_name: String::from("docker-compose.yml"),
            working_dir: settings.get("working_dir").unwrap_or(&String::new()).clone()
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

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, String> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        // TODO: Check for docker-compose version for a more controlled approach?
        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "10") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "8") {
            // Docker API is much better suited for this than using the docker-compose CLI. More effective too.
            // TODO: find down-status compose-projects with find-command?
            command.arguments(vec!["curl", "--unix-socket", "/var/run/docker.sock", "http://localhost/containers/json?all=true"]);
            Ok(command.to_string())
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.return_code == 7 {
            // Coudldn't connect. Daemon is probably not present.
            return Ok(DataPoint::empty());
        }

        let mut containers: Vec<ContainerDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;
        containers.retain(|container| container.labels.contains_key("com.docker.compose.config-hash"));

        // There will be 2 levels of multivalues (services under projects).
        let mut projects_datapoint = DataPoint::empty();

        // Group containers by project name.
        let mut projects = HashMap::<String, Vec<DataPoint>>::new();

        for container in containers {
            let project = match container.labels.get("com.docker.compose.project") {
                Some(project) => project.clone(),
                None => {
                    // Likely a container that is not used with docker-compose.
                    log::info!("Container {} has no com.docker.compose.project label and therefore can't be used", container.id);
                    continue;
                }
            };

            if !projects.contains_key(&project) {
                projects.insert(project.clone(), Vec::new());
            }

            let working_dir = match container.labels.get("com.docker.compose.project.working_dir") {
                Some(working_dir) => working_dir.clone(),
                None => {
                    log::warn!("Container {} has no com.docker.compose.project.working_dir label set.", container.id);
                    if !self.working_dir.is_empty() {
                        let working_dir = format!("{}/{}", self.working_dir, project);
                        log::warn!("User-defined working_dir \"{}\" is used instead. It isn't guaranteed that this is correct.", working_dir);
                        working_dir
                    }
                    else {
                        // Some earlier Docker Compose versions don't include this label.
                        log::error!("working_dir setting has to be set in module settings with this version of Docker Compose.");
                        continue;
                    }
                }
            };

            let service = container.labels.get("com.docker.compose.service").unwrap().clone();
            let compose_file = Path::new(&working_dir)
                                    .join(&self.compose_file_name).to_string_lossy().to_string();

            let mut data_point = DataPoint::labeled_value_with_level(service.clone(), container.status.to_string(), container.state.to_criticality());
            data_point.description = container.image.clone();
            data_point.command_params = vec![compose_file, service];

            projects.get_mut(&project).unwrap().push(data_point);
        }

        let mut projects_sorted = projects.keys().cloned().collect::<Vec<String>>();
        projects_sorted.sort();

        for project in projects_sorted {
            let mut data_points = projects.remove_entry(&project).unwrap().1;
            data_points.sort_by(|left, right| left.label.cmp(&right.label));

            let compose_file = match data_points.first() {
                Some(first) => first.command_params[0].clone(),
                None => { log::error!("No compose-file found for project {}", project); continue; }
            };

            // Check just in case that all have the same compose-file.
            if data_points.iter().any(|point| point.command_params[0] != compose_file) {
                panic!("Containers under same project can't have different compose-files");
            }

            let most_critical = data_points.iter().max_by_key(|datapoint| datapoint.criticality).unwrap();
            let mut services_datapoint = DataPoint::labeled_value_with_level(project.clone(), most_critical.value.clone(), most_critical.criticality);
            services_datapoint.command_params = vec![compose_file];
            services_datapoint.multivalue = data_points;

            projects_datapoint.multivalue.push(services_datapoint);
        }

        Ok(projects_datapoint)
    }
}