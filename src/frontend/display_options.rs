use serde_derive::{Serialize, Deserialize};
use crate::module::command::UIAction;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DisplayOptions {
    pub display_style: DisplayStyle,
    /// Text to display in front of the value.
    /// For multi-values this gets displayed (not always displayed) as a header above the list of values.
    // TODO: validate that this is always provided?
    pub display_text: String,
    // TODO: validate for alphanumeric chars?
    pub display_icon: String,

    /// Category for monitor or command. Monitors and commands in same category are grouped to the same box.
    // TODO: validate that this is always provided?
    pub category: String,
    pub unit: String,

    /// For monitors that produce a group of values.
    pub use_multivalue: bool,
    pub ignore_from_summary: bool,

    /// Display confirmation dialog with this text first.
    pub confirmation_text: String,

    /// Monitor id to attach commands to, instead of displaying on just category-level.
    pub parent_id: String,

    /// For multi-level multivalues. Limits this command to specific level so it's not displayed on every line.
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
}

#[derive(Clone, Serialize, Deserialize)]
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
