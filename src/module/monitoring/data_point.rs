/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashSet;
use std::fmt;
use serde::{Serialize, Deserialize};
use crate::enums::Criticality;

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DataPoint {
    /// With multivalue, value can be a composite result/value of all of the values.
    /// For example, with service statuses, this can show the worst state in the multivalue group.
    pub value: String,
    /// Pure integer value, currently used with charts and progress bars (DisplayStyle::ProgressBar).
    pub value_float: f32,
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
    #[serde(default, skip_serializing_if = "is_false")]
    pub acknowledged: bool,
}

impl DataPoint {
    pub fn new<Stringable: ToString>(value: Stringable) -> Self {
        DataPoint {
            value: value.to_string(),
            criticality: Criticality::Normal,
            ..Default::default()
        }
    }

    pub fn label<Stringable: ToString>(label: Stringable) -> Self {
        DataPoint {
            value: " ".to_string(),
            label: label.to_string(),
            criticality: Criticality::Normal,
            ..Default::default()
        }
    }

    pub fn labeled_value<Stringable: ToString>(label: Stringable, value: Stringable) -> Self {
        DataPoint {
            value: value.to_string(),
            label: label.to_string(),
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

    pub fn not_available(message: &str) -> Self {
        DataPoint {
            label: message.to_string(),
            criticality: Criticality::NotAvailable,
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty() && self.multivalue.is_empty()
    }

    pub fn empty() -> Self {
        Default::default()
    }

    pub fn pending() -> Self {
        DataPoint {
            criticality: Criticality::NoData,
            ..Default::default()
        }
    }

    pub fn invalid_response() -> Self {
        DataPoint {
            value: "Invalid response".to_string(),
            criticality: Criticality::Error,
            ..Default::default()
        }
    }

    pub fn empty_and_critical() -> Self {
        DataPoint {
            criticality: Criticality::Critical,
            ..Default::default()
        }
    }

    pub fn with_description<Stringable: ToString>(mut self, description: Stringable) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn is_platform_info(&self) -> bool {
        self.value == "_platform_info"
    }

    /// Raises this point's criticality to the max among children.
    /// Used by monitor builders while assembling the tree.
    pub fn update_criticality_from_children(&mut self) {
        if let Some(criticality) = self.multivalue.iter().map(|datapoint| datapoint.criticality).max() {
            self.criticality = std::cmp::max(self.criticality, criticality);
        }
    }

    /// Sentinel used when acknowledging a non-multivalue monitor (the monitor itself).
    pub const ROOT_ENTRY_ID: &'static str = "*";

    pub fn apply_acknowledged_entries(&mut self, acknowledged: &HashSet<String>) {
        self.reset_acknowledged();
        if self.multivalue.is_empty() {
            self.acknowledged = acknowledged.contains(Self::ROOT_ENTRY_ID)
                || (!self.label.is_empty() && acknowledged.contains(&self.label));
            return;
        }

        Self::apply_acknowledged_entries_recursive(self, acknowledged, None);
        self.recalculate_criticality_from_children();
    }

    pub fn collect_alert_leaves(&self, use_multivalue: bool, root_label: &str) -> Vec<AlertLeaf> {
        let mut leaves = Vec::new();
        if !use_multivalue {
            if Self::is_alert_leaf(self) {
                leaves.push(AlertLeaf {
                    entry_id: Self::ROOT_ENTRY_ID.to_string(),
                    label: if root_label.is_empty() { self.label.clone() } else { root_label.to_string() },
                    value: self.value.clone(),
                    criticality: self.criticality,
                    acknowledged: self.acknowledged,
                });
            }
            return leaves;
        }

        Self::collect_alert_leaves_recursive(self, None, &mut leaves);
        leaves
    }

    fn is_alert_leaf(point: &DataPoint) -> bool {
        point.criticality != Criticality::Ignore
            && (point.acknowledged || matches!(point.criticality, Criticality::Warning | Criticality::Error | Criticality::Critical))
    }

    /// Nested multivalue ids are `parent/child` paths; top-level children use the label alone.
    fn entry_path(parent_path: Option<&str>, label: &str) -> String {
        match parent_path {
            Some(parent) => format!("{}/{}", parent, label),
            None => label.to_string(),
        }
    }

    fn reset_acknowledged(&mut self) {
        self.acknowledged = false;
        for child in self.multivalue.iter_mut() {
            child.reset_acknowledged();
        }
    }

    fn apply_acknowledged_entries_recursive(point: &mut DataPoint, acknowledged: &HashSet<String>, parent_path: Option<&str>) {
        for child in point.multivalue.iter_mut() {
            let entry_id = Self::entry_path(parent_path, &child.label);
            child.acknowledged = acknowledged.contains(&entry_id);
            Self::apply_acknowledged_entries_recursive(child, acknowledged, Some(&entry_id));
            if !child.multivalue.is_empty() {
                child.recalculate_criticality_from_children();
            }
        }
    }

    fn collect_alert_leaves_recursive(point: &DataPoint, parent_path: Option<&str>, leaves: &mut Vec<AlertLeaf>) {
        for child in point.multivalue.iter() {
            if child.criticality == Criticality::Ignore {
                continue;
            }

            let entry_id = Self::entry_path(parent_path, &child.label);

            if child.multivalue.is_empty() {
                if Self::is_alert_leaf(child) {
                    leaves.push(AlertLeaf {
                        entry_id: entry_id.clone(),
                        label: child.label.clone(),
                        value: child.value.clone(),
                        criticality: child.criticality,
                        acknowledged: child.acknowledged,
                    });
                }
            }
            else {
                Self::collect_alert_leaves_recursive(child, Some(&entry_id), leaves);
            }
        }
    }

    /// After ack flags change: set criticality from unacked children only.
    /// If every child is acknowledged, fall back to Normal.
    fn recalculate_criticality_from_children(&mut self) {
        let most_critical = self.multivalue.iter()
            .filter(|datapoint| !datapoint.acknowledged)
            .map(|datapoint| datapoint.criticality)
            .max();

        match most_critical {
            Some(criticality) => self.criticality = criticality,
            None if !self.multivalue.is_empty() => self.criticality = Criticality::Normal,
            None => {},
        }
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
            value_float: 0.0,
            acknowledged: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AlertLeaf {
    pub entry_id: String,
    pub label: String,
    pub value: String,
    pub criticality: Criticality,
    pub acknowledged: bool,
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
