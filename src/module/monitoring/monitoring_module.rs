
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
    fn refresh(&mut self, host: &Host, connection: &mut Box<dyn ConnectionModule>) -> Result<MonitoringData, String>;
    fn get_connector_spec(&self) -> ModuleSpecification {
        ModuleSpecification::empty()
    }

    fn new_monitoring_module(settings: &HashMap<String, String>) -> Box<dyn MonitoringModule> where Self: Sized + 'static {
        Box::new(Self::new(settings))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Criticality {
    Normal,
    Warning,
    Error,
    Critical,
}

pub struct MonitoringData {
    pub value: String,
    pub multivalue: Vec<MonitoringData>,
    pub unit: String,
    pub criticality: Criticality,
    pub time: DateTime<Utc>,
}

impl MonitoringData {
    pub fn new(value: String, unit: String) -> Self {
        MonitoringData {
            value: value,
            multivalue: Vec::new(),
            unit: unit,
            criticality: Criticality::Normal,
            time: Utc::now(),
        }
    }

    pub fn new_with_level(value: String, unit: String, criticality: Criticality) -> Self {
        MonitoringData {
            value: value,
            multivalue: Vec::new(),
            unit: unit,
            criticality: criticality,
            time: Utc::now(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty() && self.unit.is_empty() && self.multivalue.is_empty()
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

impl Default for MonitoringData {
    fn default() -> Self {
        MonitoringData {
            value: String::new(),
            multivalue: Vec::new(),
            unit: String::new(),
            criticality: Criticality::Normal,
            time: Utc::now(),
        }
    }
}

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