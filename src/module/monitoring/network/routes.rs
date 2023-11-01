
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};

use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="network-routes",
    version="0.0.1",
    description="Provides routing table information.",
    settings={
    }
)]
pub struct Routes {
}

impl Module for Routes {
    fn new(_: &HashMap<String, String>) -> Self {
        Routes {
        }
    }
}

impl MonitoringModule for Routes {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Routes"),
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
        if host.platform.version_is_same_or_greater_than(platform_info::Flavor::Debian, "8") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::Ubuntu, "18") ||
           host.platform.version_is_same_or_greater_than(platform_info::Flavor::CentOS, "7") {
            Ok(String::from("ip route ls"))
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();

        let lines = response.message.lines();
        for line in lines {
            // Get substring before word "proto".
            let route = line.split("proto").next().unwrap().trim().to_string();
            let mut parts = route.split("dev");
            let subnet = parts.next().unwrap().trim().to_string();
            let interface = parts.next().unwrap().trim().to_string();

            let data_point = DataPoint::labeled_value(subnet, interface);
            result.multivalue.push(data_point);
        }
        Ok(result)
    }
}