
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{ Host, utils::enums::Criticality, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("ssh", "0.0.1")]
pub struct Ssh;

impl Module for Ssh {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Ssh { }
    }
}

impl MonitoringModule for Ssh {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::StatusUpDown,
            display_text: String::from("SSH"),
            category: String::from("network"),
            ..Default::default()
        }
    }

    fn process_response(&self, _host: Host, _responses: ResponseMessage, connector_is_connected: bool) -> Result<DataPoint, String> {
        match connector_is_connected {
            true => {
                Ok(DataPoint::value_with_level(String::from("up"), Criticality::Normal))
            },
            false => {
                Ok(DataPoint::value_with_level(String::from("down"), Criticality::Critical))
            },
        }
    }
}