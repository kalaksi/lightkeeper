use serde_derive::Serialize;

#[derive(Clone, Serialize, Default)]
pub struct DisplayOptions {
    pub display_style: DisplayStyle,
    pub display_text: String,
    pub display_icon: String,
    pub category: String,
    pub unit: String,
    // For monitors that produce a group of values.
    pub use_multivalue: bool,
    // Optional monitor id to attach actions to, instead of displaying on category-level.
    pub parent_id: String,
}

impl DisplayOptions {
    pub fn just_style(display_style: DisplayStyle) -> Self {
        DisplayOptions {
            display_style: display_style,
            ..Default::default()
        }
    }
}

#[derive(Clone, Serialize)]
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
