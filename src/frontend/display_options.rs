use serde_derive::{Serialize, Deserialize};
use strum_macros::Display;

use crate::{
    module::command::UIAction,
    enums::Criticality,
    utils::string_validation,
};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct DisplayOptions {
    /// Action to be executed by the frontend after sending the command.
    pub action: UIAction,
    /// For multi-level multivalues. Limits this command to specific level (i.e. specific rows) so it's not displayed on every line.
    /// Default is 0 which means that this limit is disabled.
    pub multivalue_level: u8,
    pub display_style: DisplayStyle,
    /// For monitors: text to display in the left column. For multi-values this gets
    ///               displayed (not always displayed) as a header above the list of values.
    /// For commands: text to display in button tooltip.
    pub display_text: String,
    pub display_icon: String,
    /// For commands: title to display in tab or window.
    pub tab_title: String,

    /// Category for monitor or command. Monitors and commands in same category are grouped to the same box.
    pub category: String,
    pub unit: String,

    /// For monitors that produce a group of values.
    pub use_multivalue: bool,
    /// Don't include in the host table status summary.
    pub use_without_summary: bool,
    /// Use DataPoint values in graphical charts.
    pub use_with_charts: bool,
    /// extension modules. If set, will hide defined monitor id from summary.
    pub override_summary_monitor_id: String,

    /// Display confirmation dialog with this text.
    pub confirmation_text: String,

    /// If set, will prompt the user for some additional parameters before launching command.
    pub user_parameters: Vec<UserInputField>,

    /// Monitor id to attach commands to, instead of displaying on just category-level.
    pub parent_id: String,
    /// This is for command modules that want to attach to e.g. both a monitoring module and a monitoring extension module.
    /// Should be rarely needed. Not the optimal solution since there could be multiple parents.
    pub secondary_parent_id: String,

    /// Show only if related monitor's criticality is one of these.
    /// Can be used, for example, for start and stop buttons.
    pub depends_on_criticality: Vec<Criticality>,
    /// Show only if related monitor's value is one of these.
    pub depends_on_value: Vec<String>,
    /// Show only if related monitor's tags contain one of these.
    pub depends_on_tags: Vec<String>,
    pub depends_on_no_tags: Vec<String>,
}

impl DisplayOptions {
    pub fn validate(&self) -> Result<(), String> {
        if self.display_style == DisplayStyle::Icon && self.display_icon.is_empty() {
            return Err(String::from("Icon display style requires display_icon to be set."));
        }

        // Must be alphanumeric.
        if !self.display_icon.is_empty() && !string_validation::is_alphanumeric_with(&self.display_icon, "-") {
            return Err(String::from("display_icon must only contain alphanumeric characters and dashes."));
        }

        if self.display_text.is_empty() {
            return Err(String::from("display_text must be set."));
        }

        for user_input_field in &self.user_parameters {
            if user_input_field.label.is_empty() {
                return Err(String::from("User input field label must be set."));
            }

            if user_input_field.units.iter().any(|unit| !string_validation::is_alphanumeric(unit)) {
                return Err(String::from("Input field units must only contain alphanumeric characters."));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Display, PartialEq)]
pub enum DisplayStyle {
    #[default]
    Text,
    CriticalityLevel,
    Icon,
    ProgressBar,
}

#[derive(Clone, Default, Serialize, Deserialize, Display, PartialEq)]
pub enum UserInputFieldType {
    #[default]
    Text,
    Integer,
    DecimalNumber,
    Option,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct UserInputField {
    pub field_type: UserInputFieldType,
    pub label: String,
    pub default_value: String,
    pub units: Vec<String>,
    pub validator_regexp: String,
    pub additional_validator_regexp: String,
    pub options: Vec<String>,
    pub option_descriptions: Vec<String>,
}

impl UserInputField {
    pub fn number(label: &str, default_value: &str) -> Self {
        UserInputField {
            field_type: UserInputFieldType::Integer,
            label: label.to_string(),
            default_value: default_value.to_string(),
            validator_regexp: String::from("^\\d+$"),
            ..Default::default()
        }
    }

    pub fn decimal_number(label: &str, default_value: &str) -> Self {
        UserInputField {
            field_type: UserInputFieldType::DecimalNumber,
            label: label.to_string(),
            default_value: default_value.to_string(),
            validator_regexp: String::from("^\\d+(\\.\\d+)?$"),
            ..Default::default()
        }
    }

    pub fn number_with_units(label: &str, default_value: &str, units: &[&str]) -> Self {
        UserInputField {
            field_type: UserInputFieldType::Integer,
            label: label.to_string(),
            default_value: default_value.to_string(),
            validator_regexp: format!("^\\d+ ?({})$", units.join("|")),
            units: units.iter().map(ToString::to_string).collect(),
            ..Default::default()
        }
    }

    pub fn decimal_number_with_units(label: &str, default_value: &str, units: &[&str]) -> Self {
        let units = units.into_iter().map(|unit| unit.to_string()).collect::<Vec<_>>();

        UserInputField {
            field_type: UserInputFieldType::DecimalNumber,
            label: label.to_string(),
            default_value: default_value.to_string(),
            // Also allows decimal point without trailing number.
            // Otherwise, the field is too restrictive for normal use as user can't even temporarily remove numbers after the decimal point.
            validator_regexp: format!("^\\d+(\\.\\d*)? ?({})$", units.join("|")),
            additional_validator_regexp: format!("^\\d+(\\.\\d+)? ?({})$", units.join("|")),
            units: units,
            ..Default::default()
        }
    }
}