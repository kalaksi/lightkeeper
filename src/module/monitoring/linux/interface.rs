
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::DataPoint,
};

#[derive(Clone)]
pub struct Interface {
    ignored_interfaces: Vec<String>,
}

impl Module for Interface {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("interface", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Interface {
            ignored_interfaces: vec![
                String::from("br-"),
                String::from("docker"),
                String::from("lo")
            ]
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Interface {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

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

    fn get_connector_message(&self) -> String {
        String::from("ip -o addr show")
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();

        let lines = response.message.split('\n');
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