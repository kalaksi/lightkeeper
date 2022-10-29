use std::str::FromStr;
use std::default::Default;
use std::fmt::Display;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum HostStatus {
    Up,
    Down,
}

impl FromStr for HostStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match String::from(s).to_lowercase().as_str() {
            "up" => Ok(HostStatus::Up),
            _ => Ok(HostStatus::Down),
        }
    }
}

impl Default for HostStatus {
    fn default() -> Self {
        HostStatus::Down
    }
}

impl Display for HostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostStatus::Up => write!(f, "up"),
            HostStatus::Down => write!(f, "down"),
        }
    }
}


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