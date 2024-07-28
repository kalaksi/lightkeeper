
use std::collections::HashMap;

use lightkeeper_module::monitoring_module;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="kernel",
    version="0.0.1",
    description="Provides kernel version and architecture information.",
)]
pub struct Kernel;

impl Module for Kernel {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Kernel { }
    }
}

impl MonitoringModule for Kernel {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Kernel version"),
            category: String::from("host"),
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("uname -r -m"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        Ok(DataPoint::new(response.message.replace(" ", " (") + ")"))
    }
}