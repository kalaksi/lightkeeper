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

    /// Monitor id to attach commands to, instead of displaying on just category-level.
    pub parent_id: String,

    /// Show only if related monitor's criticality is one of these.
    /// Can be used, for example, for start and stop buttons.
    pub depends_on_criticality: Vec<Criticality>,
    /// Show only if related monitor's value is one of these.
    pub depends_on_value: Vec<String>,

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
        if !self.display_icon.is_empty() && !string_validation::is_alphanumeric_with_dash(&self.display_icon) {
            return Err(String::from("display_icon must only contain alphanumeric characters and dashes."));
        }

        if self.display_text.is_empty() {
            return Err(String::from("display_text must be set."));
        }

        if self.category.is_empty() {
            return Err(String::from("Category must be set."));
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
