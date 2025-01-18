
use std::collections::HashMap;
use crate::enums::Criticality;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::module::platform_info;
use crate::{
    Host,
    frontend,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="load",
    version="0.0.1",
    description="Provides information about average load (using uptime-command).",
)]
pub struct Load;

impl Module for Load {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Load { }
    }
}

impl MonitoringModule for Load {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Loads"),
            category: String::from("host"),
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, LkError> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("uptime"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        if response.is_error() {
            return Ok(DataPoint::value_with_level(response.message, Criticality::Critical))
        }

        let parts = response.message.split("load average: ").collect::<Vec<&str>>();
        Ok(DataPoint::new(parts[1].to_string()))
    }
}