
use std::collections::HashMap;
use crate::enums::criticality::Criticality;
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
    name="uptime",
    version="0.0.1",
    description="Gets host uptime in days.",
)]
pub struct Uptime;

impl Module for Uptime {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Uptime { }
    }
}

impl MonitoringModule for Uptime {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Uptime"),
            category: String::from("host"),
            unit: String::from("days"),
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
            return Err(response.message);
        }

        if let Some((_, tail)) = response.message.split_once("up ") {
            if let Some((uptime, _)) = tail.split_once(",") {
                if let Some((days, _)) = uptime.split_once(" day") {
                    if let Ok(days) = days.trim().parse::<f64>() {
                        return Ok(DataPoint::new(days.to_string()));
                    }
                }
                else {
                    return Ok(DataPoint::new("0"));
                }
            }
        }

        Ok(DataPoint::value_with_level(response.message, Criticality::Critical))
    }
}