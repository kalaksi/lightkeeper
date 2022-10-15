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

    fn get_connector_message(&self) -> String {
        String::from("")
    }

    fn process_response(&self, host: Host, response: ResponseMessage, connector_is_connected: bool) -> Result<DataPoint, String>;

}

#[derive(Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    pub values: Vec<DataPoint>,
    pub display_options: DisplayOptions,
    pub is_critical: bool,
}

impl MonitoringData {
    pub fn new(display_options: DisplayOptions) -> Self {
        MonitoringData {
            values: Vec::new(),
            display_options: display_options,
            is_critical: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataPoint {
    // With multivalue, value can be a composite result/value of all of the values.
    // For example, with service statuses, this can show the worst state in the multivalue group.
    pub value: String,
    // Optional. Can be used for additional labeling of the value. Useful especially with multivalues.
    pub label: String,
    // Optional identifer. Can be used as a command parameter.
    // Some commands will require this in which case the parent monitor has to populate this.
    pub source_id: String,
    pub multivalue: Vec<DataPoint>,
    pub criticality: Criticality,
    pub time: DateTime<Utc>,
}

impl DataPoint {
    pub fn new(value: String) -> Self {
        DataPoint {
            value: value,
            label: String::from(""),
            source_id: String::from(""),
            multivalue: Vec::new(),
            criticality: Criticality::Normal,
            time: Utc::now(),
        }
    }

    pub fn new_with_level(value: String, criticality: Criticality) -> Self {
        DataPoint {
            value: value,
            label: String::from(""),
            source_id: String::from(""),
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
        let mut empty = Self::default();
        empty.criticality = Criticality::Critical;
        empty
    }
}

impl Default for DataPoint {
    fn default() -> Self {
        DataPoint {
            value: String::from(""),
            label: String::from(""),
            source_id: String::from(""),
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
