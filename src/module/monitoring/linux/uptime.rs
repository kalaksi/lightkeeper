
use std::collections::HashMap;
use chrono::{ NaiveDateTime, Utc };
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
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, String> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("uptime -s"))
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        let boot_datetime = NaiveDateTime::parse_from_str(&response.message, "%Y-%m-%d %H:%M:%S")
                                        .map_err(|e| e.to_string())?;

        let uptime = Utc::now().naive_utc() - boot_datetime;
        Ok(DataPoint::new(uptime.num_days().to_string()))
    }
}