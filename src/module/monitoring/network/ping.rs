
use std::collections::HashMap;
use oping;
use crate::{ Host, utils::enums::Criticality, frontend };
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::DataPoint,
};


#[derive(Clone)]
pub struct Ping;

impl Module for Ping {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("ping", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Ping { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Ping {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_name: String::from("Ping"),
            display_style: frontend::DisplayStyle::String,
            category: String::from("network"),
            unit: String::from("ms"),
            ..Default::default()
        }
    }

    fn process_response(&self, host: Host, _response: String, _connector_is_connected: bool) -> Result<DataPoint, String> {
        let mut ping = oping::Ping::new();

        ping.set_timeout(5.0)
            .map_err(|e| e.to_string())?;

        ping.add_host(host.ip_address.to_string().as_str())
            .map_err(|e| e.to_string())?;

        let mut responses = ping.send()
                                .map_err(|e| e.to_string())?;

        let response = responses.next().unwrap();

        if response.latency_ms < 0.0 {
            Ok(DataPoint::new_with_level(String::from("-"), Criticality::Critical))
        }
        else {
            Ok(DataPoint::new_with_level(response.latency_ms.to_string(), Criticality::Normal))
        }

    }
}