
use std::collections::HashMap;
use crate::Host;
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::Criticality,
    monitoring::DisplayStyle,
    monitoring::DisplayOptions,
    monitoring::DataPoint,
};

#[derive(Clone)]
pub struct Ssh;

impl Module for Ssh {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("ssh", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Ssh { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Ssh {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> DisplayOptions {
        DisplayOptions {
            display_name: String::from("SSH"),
            display_style: DisplayStyle::StatusUpDown,
            category: String::from("network"),
            use_multivalue: false,
            unit: String::from(""),
        }
    }

    fn process_response(&self, _host: Host, _response: String, connector_is_connected: bool) -> Result<DataPoint, String> {
        match connector_is_connected {
            true => {
                Ok(DataPoint::new_with_level(String::from("up"), Criticality::Normal))
            },
            false => {
                Ok(DataPoint::new_with_level(String::from("down"), Criticality::Critical))
            },
        }
    }
}