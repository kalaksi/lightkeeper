
use std::collections::HashMap;
use oping;
use crate::Host;
use crate::module::{
    Module,
    Metadata,
    connection::ConnectionModule,
    monitoring::MonitoringModule,
    monitoring::DataPoint,
    monitoring::DisplayOptions,
    monitoring::DisplayStyle,
    monitoring::Criticality,
    ModuleSpecification,
};


pub struct Ping;

impl Module for Ping {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from("ping"), String::from("0.0.1")),
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
    fn get_display_options(&self) -> DisplayOptions {
        DisplayOptions {
            display_style: DisplayStyle::String,
            display_name: String::from("Ping"),
            use_multivalue: false,
            unit: String::from("ms"),
        }
    }

    fn refresh(&mut self, host: &Host, _connection: &mut Box<dyn ConnectionModule>) -> Result<DataPoint, String> {
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