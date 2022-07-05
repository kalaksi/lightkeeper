use serde_derive::Serialize;

#[derive(Clone, Serialize, Default)]
pub struct DisplayOptions {
    pub display_name: String,
    pub display_style: DisplayStyle,
    pub category: String,
    pub unit: String,
    // For monitors that produce a group of values.
    pub use_multivalue: bool,
    pub parent_id: String,
}

impl DisplayOptions {
    pub fn just_style(display_style: DisplayStyle) -> Self {
        DisplayOptions {
            display_name: String::from(""),
            display_style: display_style,
            category: String::from(""),
            unit: String::from(""),
            use_multivalue: false,
            parent_id: String::from(""),
        }
    }
}

#[derive(Clone, Serialize)]
pub enum DisplayStyle {
    String,
    StatusUpDown,
    CriticalityLevel,
}

impl Default for DisplayStyle {
    fn default() -> Self {
        DisplayStyle::String
    }
}
