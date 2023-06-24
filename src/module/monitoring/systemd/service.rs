
use serde_derive::Deserialize;
use serde_json;
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};

use crate::enums;
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;
use crate::utils::ShellCommand;

#[monitoring_module("systemd-service", "0.0.1")]
pub struct Service {
    included_services: Vec<String>,
    excluded_services: Vec<String>,
}

impl Module for Service {
    fn new(settings: &HashMap<String, String>) -> Self {
        Service {
            included_services: settings.get("included_services").unwrap_or(&String::from(""))
                                       .split(',')
                                       .filter(|value| !value.is_empty())
                                       .map(|value| value.to_string())
                                       .collect(),
            excluded_services: settings.get("excluded_services").unwrap_or(&String::from(""))
                                       .split(',')
                                       .filter(|value| !value.is_empty())
                                       .map(|value| value.to_string())
                                       .collect(),
        }
    }
}

impl MonitoringModule for Service {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Services"),
            category: String::from("systemd"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, String> {
        let mut command = ShellCommand::new();

        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "10") {
            command.arguments(vec!["busctl", "--no-pager", "--json=short", "call", "org.freedesktop.systemd1",
                                    "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "ListUnits"]);
            Ok(command.to_string())
        }
        else if host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "8") {
            command.arguments(vec!["busctl", "--no-pager", "call", "org.freedesktop.systemd1",
                                    "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "ListUnits"]);
            Ok(command.to_string())
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {

        let mut result = DataPoint::empty();

        let all_units = if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "10") ||
                           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "20") {

            let mut response: DbusResponse = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;
            response.data.remove(0)
        }
        else if host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "8") {
            // First 2 words are like: a(ssssssouso) 253 
            let actual_content = response.message.splitn(3, " ").last().unwrap_or("");
            // Removes first and last quotes and then splits.
            let fields: Vec<&str> = actual_content[1..actual_content.len() -1]
                .split("\" ")
                .map(|field| field.trim_matches('"'))
                .collect();

            fields.chunks_exact(9).map(|chunk| {
                UnitData {
                    id: chunk[0].to_string(),
                    name: chunk[1].to_string(),
                    load_state: chunk[2].to_string(),
                    active_state: chunk[3].to_string(),
                    sub_state: chunk[4].to_string(),
                    _follows: chunk[5].to_string(),
                    _unit_path: chunk[6].to_string(),
                    // Previous parsing misses unquoted integers.
                    _job_id: chunk[7].split(" ").next().unwrap_or("0").parse().unwrap(),
                    _job_type: chunk[7].split(" \"").nth(1).unwrap_or("").to_string(),
                    _job_path: chunk[8].to_string(),
                }
            }).collect::<Vec<UnitData>>()
        }
        else {
            Vec::new()
        };

        let services = all_units.iter().filter(|unit| unit.id.ends_with(".service"));
        let mut allowed_services = services
            .filter(|unit| self.included_services.is_empty() || self.included_services.iter().any(|id| unit.id.starts_with(id)))
            .filter(|unit| self.excluded_services.is_empty() || !self.excluded_services.iter().any(|id| unit.id.starts_with(id)))
            .collect::<Vec<&UnitData>>();

        // Sort alphabetically by ID.
        allowed_services.sort_by_key(|unit| &unit.id);


        result.multivalue = allowed_services.iter().map(|unit| {
            let mut point = DataPoint::labeled_value(unit.id.clone(), unit.sub_state.clone());
            point.description = unit.name.clone();

            // Add some states as tags for the UI.
            if ["masked"].contains(&unit.load_state.as_str()) {
                point.tags.push(unit.load_state.clone());
            }

            match unit.sub_state.as_str() {
                "dead" => {
                    point.criticality = enums::Criticality::Critical;
                },
                "exited" => {
                    if unit.active_state == "active" {
                        point.criticality = enums::Criticality::Normal;
                        point.value = format!("{} ({})", unit.sub_state.clone(), unit.active_state.clone());
                    }
                    else {
                        point.criticality = enums::Criticality::Warning;
                    }
                },
                "running" => {
                    point.criticality = enums::Criticality::Normal;
                },
                _ => {
                    point.criticality = enums::Criticality::Warning;
                }
            };

            point.command_params.push(unit.id.clone());
            point
        }).collect();

        if !result.multivalue.is_empty() {
            let most_critical = result.multivalue.iter().max_by_key(|value| value.criticality).unwrap();
            result.criticality = most_critical.criticality;
        }

        Ok(result)
    }
}

// For deserializing the busctl output.
#[derive(Deserialize)]
struct DbusResponse {
    // type: String,
    data: Vec<Vec<UnitData>>
}

#[derive(Deserialize)]
struct UnitData {
    id: String,
    name: String,
    load_state: String,
    active_state: String,
    sub_state: String,
    _follows: String,
    _unit_path: String,
    _job_id: u32,
    _job_type: String,
    _job_path: String,
}