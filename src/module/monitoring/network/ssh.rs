
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{ Host, enums::Criticality, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="ssh",
    version="0.0.1",
    description="DEPRECATED. Checks if the SSH service is available.",
)]
pub struct Ssh;

impl Module for Ssh {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Ssh { }
    }
}

impl MonitoringModule for Ssh {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("SSH"),
            category: String::from("network"),
            ..Default::default()
        }
    }

    fn process_response(&self, host: Host, _responses: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if !host.platform.is_set() {
            Ok(DataPoint::value_with_level(String::from("down"), Criticality::Critical))
        }
        else {
            Ok(DataPoint::value_with_level(String::from("up"), Criticality::Normal))
        }
    }
}