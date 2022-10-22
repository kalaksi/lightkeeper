
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
pub struct Kernel;

impl Module for Kernel {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("kernel", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Kernel { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Kernel {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

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

    fn get_connector_message(&self) -> String {
        String::from("uname -r -m")
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        Ok(DataPoint::new(response.message.replace(" ", " (") + ")"))
    }
}