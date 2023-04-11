
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
        // TODO: use dbus?
        if host.platform.is_newer_than(platform_info::Flavor::Debian, "8") {
            String::from("systemctl list-units -t service --all --output json")
        }
        else {
            String::new()
        }
        // Alternative:
        // String::from("systemctl list-units -t service --full --all --plain --no-legend")
    }

    fn process_response(&self, host: Host, response: ResponseMessage) -> Result<DataPoint, String> {
        if host.platform.is_newer_than(platform_info::Flavor::Debian, "8") {
            let services: Vec<ServiceUnit> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

            let mut result = DataPoint::empty();

            result.multivalue = services.iter().map(|service| {
                let mut point = DataPoint::labeled_value(service.unit.clone(), service.sub.clone());

                point.description = service.description.clone();

                // Add some states as tags for the UI.
                if ["masked"].contains(&service.load.as_str()) {
                    point.tags.push(service.load.clone());
                }

                point.criticality = match service.sub.as_str() {
                    "dead" => enums::Criticality::Critical,
                    "exited" => enums::Criticality::Error,
                    "running" => enums::Criticality::Normal,
                    _ => enums::Criticality::Warning,
                };

                if !self.included_services.contains(&service.unit) {
                    point.ignore();
                }

                point.command_params.push(service.unit.clone());

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

#[derive(Deserialize)]
struct ServiceUnit {
    unit: String,
    load: String,
    // active: String,
    sub: String,
    description: String,
}
