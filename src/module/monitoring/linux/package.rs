
use std::collections::HashMap;

use crate::module::connection::ResponseMessage;
use crate::module::platform_info::Flavor;
use crate::{ Host, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("package", "0.0.1")]
pub struct Package;

impl Module for Package {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Package { }
    }
}

impl MonitoringModule for Package {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Packages"),
            category: String::from("packages"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host) -> String {
        if host.platform.is_newer_than(Flavor::Debian, "8") {
            String::from("sudo apt list --upgradable")
        }
        else {
            String::new()
        }
    }

    fn process_response(&self, host: Host, response: ResponseMessage) -> Result<DataPoint, String> {
        if host.platform.is_newer_than(Flavor::Debian, "8") {
            let mut result = DataPoint::empty();
            let lines = response.message.split('\n').filter(|line| line.contains("[upgradable"));
            for line in lines {
                let mut parts = line.split_whitespace();
                let package = parts.next().unwrap().to_string();
                let package_name = package.split('/').next().unwrap().to_string();
                let new_version = parts.next().unwrap().to_string();
                // let mut old_version = parts.nth(3).unwrap().to_string();
                // Last character is ']'.
                // old_version.pop();
                
                let data_point = DataPoint::labeled_value(package_name, new_version);
                result.multivalue.push(data_point);
            }

            Ok(result)
        }
        else {
            self.error_unsupported()
        }
    }

}