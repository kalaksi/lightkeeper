use std::collections::HashMap;
use serde_derive::{Serialize, Deserialize};
use super::DataPoint;

use crate::{
    Host,
    module::module::Module,
    module::ModuleSpecification,
    module::MetadataSupport,
    module::connection::ResponseMessage,
    frontend::DisplayOptions,
    frontend::DisplayStyle,
};

pub type Monitor = Box<dyn MonitoringModule + Send + Sync>;

pub trait MonitoringModule : BoxCloneableMonitor + MetadataSupport + Module {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        None
    }

    fn new_monitoring_module(settings: &HashMap<String, String>) -> Monitor where Self: Sized + 'static + Send + Sync {
        Box::new(Self::new(settings))
    }

    fn get_display_options(&self) -> DisplayOptions {
        DisplayOptions {
            display_style: DisplayStyle::Text,
            display_text: self.get_module_spec().id,
            ..Default::default()
        }
    }

    fn get_connector_message(&self) -> String {
        String::from("")
    }

    fn get_connector_messages(&self) -> Vec<String> {
        Vec::new()
    }

    fn process_response(&self, _host: Host, _response: ResponseMessage) -> Result<DataPoint, String> {
        Err(String::from("Not implemented"))
    }

    fn process_responses(&self, _host: Host, _responses: Vec<ResponseMessage>) -> Result<DataPoint, String> {
        Err(String::from("Not implemented"))
    }

    // TODO: Include in the macro?
    fn uses_multiple_commands(&self) -> bool {
        false
    }

}

// Implemented by the macro.
pub trait BoxCloneableMonitor {
    fn box_clone(&self) -> Monitor;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    pub monitor_id: String,
    pub values: Vec<DataPoint>,
    pub display_options: DisplayOptions,
    pub is_critical: bool,
}

impl MonitoringData {
    pub fn new(monitor_id: String, display_options: DisplayOptions) -> Self {
        MonitoringData {
            monitor_id: monitor_id,
            values: Vec::new(),
            display_options: display_options,
            is_critical: false,
        }
    }
}