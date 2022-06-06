
use std::fmt;
use std::collections::HashMap;
use chrono::{ DateTime, Utc };

use crate::Host;
use crate::module::{
    module::Module,
    ModuleSpecification,
    connection::ConnectionModule
};

pub trait MonitoringModule : Module {
    fn refresh(&mut self, host: &Host, connection: &mut Box<dyn ConnectionModule>) -> Result<DataPoint, String>;
    fn get_connector_spec(&self) -> ModuleSpecification {
        ModuleSpecification::empty()
    }

    fn new_monitoring_module(settings: &HashMap<String, String>) -> Box<dyn MonitoringModule> where Self: Sized + 'static {
        Box::new(Self::new(settings))
    }

    fn get_display_options(&self) -> DisplayOptions {
        DisplayOptions {
            display_name: self.get_connector_spec().id,
            display_style: DisplayStyle::String,
            use_multivalue: false,
            unit: String::from(""),
        }
    }
}

pub struct DisplayOptions {
    pub unit: String,
    pub display_name: String,
    pub display_style: DisplayStyle,
    pub use_multivalue: bool,
}

impl DisplayOptions {
    pub fn just_style(display_style: DisplayStyle) -> Self {
        DisplayOptions {
            unit: String::from(""),
            display_name: String::from(""),
            display_style: display_style,
            use_multivalue: false
        }
    }
}

pub enum DisplayStyle {
    String,
    Numeric,
    StatusUpDown,
    CriticalityLevel,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Criticality {
    NoData,
    Normal,
    Warning,
    Error,
    Critical,
}

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
        let mut empty = Self::empty();
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

/*
impl fmt::Display for MonitoringData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "(empty)")
        }
        else if !self.multivalue.is_empty() {
            let values: Vec<String> = self.multivalue.iter().map(|m| format!("{} ({})", m.value, m.unit)).collect();
            write!(f, "{}", values.join(", "))
        }
        else {
            write!(f, "{} ({})", self.value, self.unit)
        }
    }
}
*/