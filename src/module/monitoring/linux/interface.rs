
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};

use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("interface", "0.0.1")]
pub struct Interface {
    ignored_interfaces: Vec<String>,
}

impl Module for Interface {
    fn new(settings: &HashMap<String, String>) -> Self {
        Interface {
            ignored_interfaces: settings.get("ignored_interfaces").unwrap_or(&String::from(""))
                                        .split(',')
                                        .filter(|value| !value.is_empty())
                                        .map(|value| value.to_string())
                                        .collect(),
        }
    }
}

impl MonitoringModule for Interface {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Interfaces"),
            category: String::from("network"),
            use_multivalue: true,
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, String> {
        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "9") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "8") {
            Ok(String::from("ip -o addr show"))
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();

        let lines = response.message.lines();
        for line in lines {
            let mut parts = line.split_whitespace();
            let if_name = parts.nth(1).unwrap().to_string();

            if self.ignored_interfaces.iter().any(|item| if_name.starts_with(item)) {
                continue;
            }

            let if_address = parts.nth(1).unwrap_or_default().to_string();
            let data_point = DataPoint::labeled_value(if_name, if_address);
            result.multivalue.push(data_point);

        }
        Ok(result)
    }
}