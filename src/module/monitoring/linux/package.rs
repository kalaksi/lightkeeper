
use std::collections::HashMap;

use crate::module::connection::ResponseMessage;
use crate::{ Host, frontend };
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::DataPoint,
};


#[derive(Clone)]
pub struct Package;

impl Module for Package {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("package", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Package { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Package {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Packages"),
            category: String::from("packages"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self) -> String {
        String::from("sudo apt list --upgradable")
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
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

}