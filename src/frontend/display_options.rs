use serde_derive::{Serialize, Deserialize};
use strum_macros::Display;

use crate::{
    module::command::UIAction,
    enums::Criticality,
    utils::string_validation,
};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct DisplayOptions {
    pub display_style: DisplayStyle,
    /// Text to display in front of the value.
    /// For multi-values this gets displayed (not always displayed) as a header above the list of values.
    pub display_text: String,
    pub display_icon: String,

    /// Category for monitor or command. Monitors and commands in same category are grouped to the same box.
    pub category: String,
    pub unit: String,

    /// For monitors that produce a group of values.
    pub use_multivalue: bool,
    pub ignore_from_summary: bool,

    /// Display confirmation dialog with this text.
    pub confirmation_text: String,

    pub user_parameters: Vec<UserInputField>,

    /// Monitor id to attach commands to, instead of displaying on just category-level.
    pub parent_id: String,

    /// Show only if related monitor's criticality is one of these.
    /// Can be used, for example, for start and stop buttons.
    pub depends_on_criticality: Vec<Criticality>,
    /// Show only if related monitor's value is one of these.
    pub depends_on_value: Vec<String>,
    /// Show only if related monitor's tags contain one of these.
    pub depends_on_tags: Vec<String>,
    pub depends_on_no_tags: Vec<String>,

    /// For multi-level multivalues. Limits this command to specific level (i.e. specific rows) so it's not displayed on every line.
    /// Default is 0 which means that this limit is disabled.
    pub multivalue_level: u8,

    /// Action to be executed by the frontend after sending the command.
    pub action: UIAction,
}

impl DisplayOptions {
    pub fn just_style(display_style: DisplayStyle) -> Self {
        DisplayOptions {
            display_style: display_style,
            ..Default::default()
        }
    }

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

        if self.category.is_empty() {
            return Err(String::from("Category must be set."));
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

#[derive(Clone, Serialize, Deserialize, Display, PartialEq)]
pub enum DisplayStyle {
    Text,
    StatusUpDown,
    CriticalityLevel,
    Icon,
    ProgressBar,
}

impl Default for DisplayStyle {
    fn default() -> Self {
        DisplayStyle::Text
    }
}

#[derive(Clone, Serialize, Deserialize, Display, PartialEq)]
pub enum UserInputFieldType {
    Text,
    Integer,
    DecimalNumber
}

impl Default for UserInputFieldType {
    fn default() -> Self {
        UserInputFieldType::Text
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct UserInputField {
    pub field_type: UserInputFieldType,
    pub label: String,
    pub default_value: String,
    pub units: Vec<String>,
    pub validator_regexp: String,
}

impl UserInputField {
    pub fn new<Stringable: ToString>(field_type: UserInputFieldType, label: Stringable, default_value: Stringable) -> Self {
        UserInputField {
            field_type: field_type,
            label: label.to_string(),
            default_value: default_value.to_string(),
            units: Vec::new(),
            validator_regexp: String::new(),
        }
    }

    pub fn number<Stringable: ToString>(label: Stringable, default_value: Stringable) -> Self {
        UserInputField {
            field_type: UserInputFieldType::Integer,
            label: label.to_string(),
            default_value: default_value.to_string(),
            units: Vec::new(),
            validator_regexp: String::from("^\\d+$")
        }
    }

    pub fn decimal_number<Stringable: ToString>(label: Stringable, default_value: Stringable) -> Self {
        UserInputField {
            field_type: UserInputFieldType::DecimalNumber,
            label: label.to_string(),
            default_value: default_value.to_string(),
            units: Vec::new(),
            validator_regexp: String::from("^\\d+(\\.\\d+)?$")
        }
    }

    pub fn number_with_units<Stringable: ToString>(label: Stringable, default_value: Stringable, units: Vec<String>) -> Self {
        UserInputField {
            field_type: UserInputFieldType::Integer,
            label: label.to_string(),
            default_value: default_value.to_string(),
            validator_regexp: format!("^\\d+ ?({})$", units.join("|")),
            units: units,
        }
    }

    pub fn decimal_number_with_units<Stringable: ToString>(label: Stringable, default_value: Stringable, units: Vec<String>) -> Self {
        UserInputField {
            field_type: UserInputFieldType::DecimalNumber,
            label: label.to_string(),
            default_value: default_value.to_string(),
            validator_regexp: format!("^\\d+(\\.\\d+)? ?({})$", units.join("|")),
            units: units,
        }
    }
}