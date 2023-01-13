use std::fmt::Display;
use serde_derive::{Serialize, Deserialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub enum Criticality {
    Ignore,
    Normal,
    // Info is basically Normal level but it will be displayed to user in some cases where Normal won't.
    Info,
    NoData,
    Warning,
    Error,
    Critical,
}

impl Display for Criticality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Criticality::Ignore => write!(f, "Ignore"),
            Criticality::Normal => write!(f, "Normal"),
            Criticality::Info => write!(f, "Info"),
            Criticality::NoData => write!(f, "NoData"),
            Criticality::Warning => write!(f, "Warning"),
            Criticality::Error => write!(f, "Error"),
            Criticality::Critical => write!(f, "Critical"),
        }
    }
}