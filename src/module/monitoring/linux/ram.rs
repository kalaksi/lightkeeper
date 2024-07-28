
use std::collections::HashMap;
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
    name="ram",
    version="0.0.1",
    description="Provides RAM usage information.",
)]
pub struct Ram;

impl Module for Ram {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Ram { }
    }
}

impl MonitoringModule for Ram {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("RAM usage"),
            category: String::from("host"),
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, LkError> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("free -m"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        let line = response.message.lines().filter(|line| line.contains("Mem:")).collect::<Vec<&str>>();
        let parts = line[0].split_whitespace().collect::<Vec<&str>>();

        let total = parts[1].parse::<u64>().unwrap();
        // used
        // free
        // shared
        // cache
        let available = parts[6].parse::<u64>()
            .map_err(|_| String::from("Unsupported platform"))?;

        let usage_percent = 1.0 - (available as f64 / total as f64);
        let value = format!("{} / {} M  ({:.0} %)", total - available, total, usage_percent * 100.0);
        Ok(DataPoint::new(value))
    }
}