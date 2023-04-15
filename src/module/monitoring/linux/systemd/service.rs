
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
}

impl Module for Service {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Service {
            // TODO: configurable (remember to automatically add .service suffix)
            included_services: vec![
                String::from("acpid.service"),
                String::from("cron.service"),
                String::from("collectd.service"),
                String::from("dbus.service"),
                String::from("ntp.service"),
                String::from("chrony.service"),
                String::from("systemd-journald.service"),
                String::from("containerd.service"),
                String::from("docker.service"),
            ]
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

    fn get_connector_message(&self, host: Host) -> String {
        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if host.platform.is_newer_than(platform_info::Flavor::Debian, "8") {
                command.arguments(vec!["busctl", "--no-pager", "--json=short", "call", "org.freedesktop.systemd1",
                                       "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "ListUnits"]);
            }
        }

        command.to_string()
    }

    fn process_response(&self, host: Host, response: ResponseMessage) -> Result<DataPoint, String> {
        if host.platform.is_newer_than(platform_info::Flavor::Debian, "8") {
            let response: DbusResponse = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

            let mut result = DataPoint::empty();

            let services = response.data.first().unwrap().iter().filter(|unit| unit.id.ends_with(".service"));
            let allowed_services = services.filter(|unit| self.included_services.contains(&unit.id));

            result.multivalue = allowed_services.map(|unit| {
                let mut point = DataPoint::labeled_value(unit.id.clone(), unit.sub_state.clone());
                point.description = unit.name.clone();

                // Add some states as tags for the UI.
                if ["masked"].contains(&unit.load_state.as_str()) {
                    point.tags.push(unit.load_state.clone());
                }

                point.criticality = match unit.sub_state.as_str() {
                    "dead" => enums::Criticality::Critical,
                    "exited" => enums::Criticality::Error,
                    "running" => enums::Criticality::Normal,
                    _ => enums::Criticality::Warning,
                };

                point.command_params.push(unit.id.clone());

                point
            }).collect();

            let most_critical = result.multivalue.iter().max_by_key(|value| value.criticality).unwrap();
            result.criticality = most_critical.criticality;
            Ok(result)
        }
        else {
            self.error_unsupported()
        }
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
    _active_state: String,
    sub_state: String,
    _follows: String,
    _unit_path: String,
    _job_id: u32,
    _job_type: String,
    _job_path: String,
}