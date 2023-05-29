use std::fmt;
use chrono::{DateTime, Utc};

use serde_derive::{Serialize, Deserialize};
use crate::enums::Criticality;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DataPoint {
    /// With multivalue, value can be a composite result/value of all of the values.
    /// For example, with service statuses, this can show the worst state in the multivalue group.
    pub value: String,
    /// Optional. Used with multivalue-data and usually filled programmatically.
    pub label: String,
    /// Optional description for label.
    pub description: String,
    /// Tags can be used for additional data that will be displayed alongside the value.
    pub tags: Vec<String>,
    /// This data is passed to commands. Contents depend on the monitoring module.
    /// First parameter has to contain a unique attribute (per DataPoint), e.g. container ID or service name,
    /// since it will be used for creating button identifiers. This restriction will probably change at some point.
    pub command_params: Vec<String>,
    // TODO: rename to children?
    pub multivalue: Vec<DataPoint>,
    pub criticality: Criticality,
    pub time: DateTime<Utc>,
    /// Unique invocation ID. Used by UI as an identifier for asynchronously executed requests and received results.
    pub invocation_id: u64,
    pub is_from_cache: bool,
}

impl DataPoint {
    pub fn new(value: String) -> Self {
        DataPoint {
            value: value,
            criticality: Criticality::Normal,
            ..Default::default()
        }
    }

    pub fn labeled_value(label: String, value: String) -> Self {
        DataPoint {
            value: value,
            label: label,
            criticality: Criticality::Normal,
            ..Default::default()
        }
    }

    pub fn labeled_value_with_level(label: String, value: String, criticality: Criticality) -> Self {
        DataPoint {
            value: value,
            label: label,
            criticality: criticality,
            ..Default::default()
        }
    }

    pub fn value_with_level(value: String, criticality: Criticality) -> Self {
        DataPoint {
            value: value,
            criticality: criticality,
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty() && self.multivalue.is_empty()
    }

    pub fn empty() -> Self {
        Default::default()
    }

    pub fn no_data() -> Self {
        DataPoint {
            criticality: Criticality::NoData,
            ..Default::default()
        }
    }

    pub fn empty_and_critical() -> Self {
        DataPoint {
            criticality: Criticality::Critical,
            ..Default::default()
        }
    }

    pub fn ignore(&mut self) {
        self.criticality = Criticality::Ignore;
    }
}

impl Default for DataPoint {
    fn default() -> Self {
        DataPoint {
            value: String::from(""),
            label: String::from(""),
            description: String::from(""),
            tags: Vec::new(),
            command_params: Vec::new(),
            multivalue: Vec::new(),
            criticality: Criticality::Normal,
            time: Utc::now(),
            invocation_id: 0,
            is_from_cache: false,
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
