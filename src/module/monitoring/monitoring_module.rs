use std::collections::{HashMap, VecDeque};

use serde_derive::{Deserialize, Serialize};

use super::DataPoint;
use crate::{
    error::LkError,
    frontend::DisplayOptions,
    frontend::DisplayStyle,
    module::connection::ResponseMessage,
    module::module::Module,
    module::MetadataSupport,
    module::ModuleSpecification,
    Host,
};

pub type Monitor = Box<dyn MonitoringModule + Send + Sync>;

pub trait MonitoringModule: BoxCloneableMonitor + MetadataSupport + Module {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        None
    }

    fn new_monitoring_module(settings: &HashMap<String, String>) -> Monitor
    where
        Self: Sized + 'static + Send + Sync,
    {
        Box::new(Self::new(settings))
    }

    fn get_display_options(&self) -> DisplayOptions {
        DisplayOptions {
            display_style: DisplayStyle::Text,
            display_text: self.get_module_spec().id,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _host: Host, _parent_result: DataPoint) -> Result<String, LkError> {
        Err(LkError::not_implemented())
    }

    fn get_connector_messages(&self, _host: Host, _parent_result: DataPoint) -> Result<Vec<String>, LkError> {
        Err(LkError::not_implemented())
    }

    fn process_response(&self, _host: Host, _response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        Err(String::new())
    }

    /// Note that if implementing this method, you will need to set is_from_cache yourself.
    fn process_responses(&self, _host: Host, _responses: Vec<ResponseMessage>, _parent_result: DataPoint) -> Result<DataPoint, String> {
        Err(String::new())
    }
}

// Implemented by the macro.
pub trait BoxCloneableMonitor {
    fn box_clone(&self) -> Monitor;
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    pub monitor_id: String,
    pub values: VecDeque<DataPoint>,
    pub display_options: DisplayOptions,
    pub is_critical: bool,
}

impl MonitoringData {
    pub fn new(monitor_id: String, display_options: DisplayOptions) -> Self {
        MonitoringData {
            monitor_id: monitor_id,
            values: VecDeque::new(),
            display_options: display_options,
            is_critical: false,
        }
    }
}
