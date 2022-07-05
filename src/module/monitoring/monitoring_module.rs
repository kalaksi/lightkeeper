use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use serde_derive::Serialize;

use crate::{
    Host,
    utils::enums::Criticality,
    module::module::Module,
    module::ModuleSpecification,
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
            display_name: self.get_module_spec().id,
            display_style: DisplayStyle::String,
            ..Default::default()
        }
    }

    fn get_connector_message(&self) -> String {
        String::from("")
    }

    fn process_response(&self, host: Host, response: String, connector_is_connected: bool) -> Result<DataPoint, String>;

}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
pub struct DataPoint {
    pub value: String,
    pub multivalue: Vec<DataPoint>,
    pub criticality: Criticality,
    pub time: DateTime<Utc>,
}

impl DataPoint {
    pub fn new(value: String) -> Self {
        DataPoint {
            value: value,
            multivalue: Vec::new(),
            criticality: Criticality::Normal,
            time: Utc::now(),
        }
    }

    pub fn new_with_level(value: String, criticality: Criticality) -> Self {
        DataPoint {
            value: value,
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
            value: String::new(),
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
