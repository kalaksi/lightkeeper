use serde_derive::{Serialize, Deserialize};
use crate::module::command::CommandAction;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DisplayOptions {
    pub display_style: DisplayStyle,
    pub display_text: String,
    pub display_icon: String,

    // Display priority controls which items are shown first. Smaller number wins.
    // TODO: formulate a better rule for using this.
    // TODO: rename to just priority
    pub display_priority: i8,
    // Category for monitor or command. Monitors and commands in same category are grouped to the same box.
    pub category: String,
    pub unit: String,

    // For monitors that produce a group of values.
    pub use_multivalue: bool,

    // Display confirmation dialog with this text first.
    pub confirmation_text: String,

    // Monitor id to attach commands to, instead of displaying on just category-level.
    pub parent_id: String,

    // Action to be executed by the frontend after sending the command.
    pub action: CommandAction,
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
}

impl Default for DisplayStyle {
    fn default() -> Self {
        DisplayStyle::Text
    }
}
