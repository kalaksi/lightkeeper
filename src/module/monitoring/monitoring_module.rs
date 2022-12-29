use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use serde_derive::{Serialize, Deserialize};

use crate::{
    Host,
    utils::enums::Criticality,
    module::module::Module,
    module::ModuleSpecification,
    module::connection::ResponseMessage,
    frontend::DisplayOptions,
    frontend::DisplayStyle,
};

pub type Monitor = Box<dyn MonitoringModule + Send + Sync>;

pub trait MonitoringModule : Module {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        None
    }

    fn new_monitoring_module(settings: &HashMap<String, String>) -> Monitor where Self: Sized + 'static + Send + Sync {
        Box::new(Self::new(settings))
    }

    // TODO: less boilerplate for module implementation?
    fn clone_module(&self) -> Monitor;

    fn get_display_options(&self) -> DisplayOptions {
        DisplayOptions {
            display_style: DisplayStyle::Text,
            display_text: self.get_module_spec().id,
            ..Default::default()
        }
    }

    // TODO: use RequestMessage?
    fn get_connector_message(&self) -> String {
        String::from("")
    }

    fn get_connector_messages(&self) -> Vec<String> {
        Vec::new()
    }

    fn process_response(&self, _host: Host, _response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        Err(String::from("Not implemented"))
    }

    fn process_responses(&self, _host: Host, _responses: Vec<ResponseMessage>, _connector_is_connected: bool) -> Result<DataPoint, String> {
        Err(String::from("Not implemented"))
    }

    // TODO: Maybe change DisplayOptions to ModuleOptions and include there?
    fn uses_multiple_commands(&self) -> bool {
        false
    }

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

// TODO: split to separate file.
#[derive(Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// With multivalue, value can be a composite result/value of all of the values.
    /// For example, with service statuses, this can show the worst state in the multivalue group.
    pub value: String,
    /// Optional. Used with multivalue-data and usually filled programmatically.
    pub label: String,
    /// This data is passed to commands. Usually contains an identifier of the source of this data,
    /// e.g. container ID or service name, so that attached commands can target the correct identity.
    pub command_params: Vec<String>,
    pub multivalue: Vec<DataPoint>,
    pub criticality: Criticality,
    pub time: DateTime<Utc>,
}

impl DataPoint {
    pub fn new(value: String) -> Self {
        DataPoint {
            value: value,
            label: String::from(""),
            command_params: Vec::new(),
            multivalue: Vec::new(),
            criticality: Criticality::Normal,
            time: Utc::now(),
        }
    }

    pub fn labeled_value(label: String, value: String) -> Self {
        DataPoint {
            value: value,
            label: label,
            command_params: Vec::new(),
            multivalue: Vec::new(),
            criticality: Criticality::Normal,
            time: Utc::now(),
        }
    }

    pub fn labeled_value_with_level(label: String, value: String, criticality: Criticality) -> Self {
        DataPoint {
            value: value,
            label: label,
            command_params: Vec::new(),
            multivalue: Vec::new(),
            criticality: criticality,
            time: Utc::now(),
        }
    }

    pub fn value_with_level(value: String, criticality: Criticality) -> Self {
        DataPoint {
            value: value,
            label: String::from(""),
            command_params: Vec::new(),
            multivalue: Vec::new(),
            criticality: criticality,
            time: Utc::now(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty() && self.multivalue.is_empty()
    }

    pub fn empty() -> Self {
        Default::default()
    }

    pub fn empty_and_critical() -> Self {
        DataPoint {
            criticality: Criticality::Critical,
            ..Default::default()
        }
    }

    pub fn hide(&mut self) {
        self.criticality = Criticality::Ignore;
    }

    pub fn none() -> Self {
        DataPoint {
            value: String::from(" "),
            ..Default::default()
        }
    }
}

impl Default for DataPoint {
    fn default() -> Self {
        DataPoint {
            value: String::from(""),
            label: String::from(""),
            command_params: Vec::new(),
            multivalue: Vec::new(),
            criticality: Criticality::Normal,
            time: Utc::now(),
        }
    }
}

impl fmt::Display for DataPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "(empty)")
        }
        else if !self.multivalue.is_empty() {
            let values: Vec<String> = self.multivalue.iter().map(|m| format!("{}", m.value)).collect();
            write!(f, "{}", values.join(", "))
        }
        else {
            write!(f, "{}", self.value)
        }
    }
}
