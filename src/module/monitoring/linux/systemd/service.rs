
use serde_derive::Deserialize;
use serde_json;
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};

use crate::utils::enums;
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::DataPoint,
};

#[derive(Clone)]
pub struct Service {
    included_services: Vec<String>,
}

impl Module for Service {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("systemd-service", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Service {
            // TODO: configurable (remember to automatically add .service suffix)
            included_services: vec![
                String::from("acpid.service"),
                String::from("cron.service"),
                String::from("dbus.service"),
                String::from("ntp.service"),
                String::from("chrony.service"),
                String::from("systemd-journald.service"),
                String::from("containerd.service"),
                String::from("docker.service"),
            ]
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Service {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Services"),
            category: String::from("systemd"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self) -> String {
        // Alternative:
        // String::from("systemctl list-units -t service --full --all --plain --no-legend")
        String::from("systemctl list-units -t service --all --output json")
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        let services: Vec<ServiceUnit> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

        let mut result = DataPoint::empty();

        result.multivalue = services.iter()
                                    .filter(|service| self.included_services.contains(&service.unit))
                                    .map(|service| {

            let mut point = DataPoint::labeled_value(service.unit.clone(), service.sub.clone());
            point.criticality = match service.sub.as_str() {
                "dead" => enums::Criticality::Critical,
                "exited" => enums::Criticality::Error,
                "running" => enums::Criticality::Normal,
                _ => enums::Criticality::Warning,
            };

            point
        }).collect();

        let most_critical = result.multivalue.iter().max_by_key(|value| value.criticality).unwrap();
        result.criticality = most_critical.criticality;
        Ok(result)
    }
}

#[derive(Deserialize)]
struct ServiceUnit {
    unit: String,
    // load: String,
    // active: String,
    sub: String,
    // description: String,
}
