
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::module::platform_info;
use crate::{
    Host,
    frontend,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("ram", "0.0.1")]
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

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, String> {
        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "10") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "8") {
            Ok(String::from("free -m"))
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "10") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "8") {

            let line = response.message.lines().filter(|line| line.contains("Mem:")).collect::<Vec<&str>>();
            let parts = line[0].split_whitespace().collect::<Vec<&str>>();

            let total = parts[1].parse::<u64>().unwrap();
            // used
            // free
            // shared
            // cache
            let available = parts[6].parse::<u64>().unwrap();

            let usage_percent = (1.0 - (available as f64 / total as f64));
            let value = format!("{} / {} M  ({:.0} %)", available, total, usage_percent * 100.0);
            Ok(DataPoint::new(value))
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }
}