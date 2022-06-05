
use std::collections::HashMap;
use crate::Host;
use crate::module::{
    Module,
    Metadata,
    connection::ConnectionModule,
    monitoring::{ MonitoringModule, Criticality, DisplayStyle, DisplayOptions, DataPoint },
    ModuleSpecification,
};

pub struct Ssh;

impl Module for Ssh {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from("ssh"), String::from("0.0.1")),
            display_name: String::from("SSH"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(settings: &HashMap<String, String>) -> Self {
        Ssh { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Ssh {
    fn get_connector_spec(&self) -> ModuleSpecification {
        ModuleSpecification::new(String::from("ssh"), String::from("0.0.1"))
    }

    fn get_display_options(&self) -> DisplayOptions {
        DisplayOptions {
            display_style: DisplayStyle::StatusUpDown,
            use_multivalue: false,
            unit: String::from(""),
        }
    }


    fn refresh(&mut self, _host: &Host, connection: &mut Box<dyn ConnectionModule>) -> Result<DataPoint, String> {
        match &connection.is_connected() {
            true => {
                Ok(DataPoint::new_with_level(String::from("up"), Criticality::Normal))
            },
            false => {
                Ok(DataPoint::new_with_level(String::from("down"), Criticality::Critical))
            },
        }
    }
}