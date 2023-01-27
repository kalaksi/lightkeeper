use std::str::FromStr;
use std::default::Default;
use std::fmt::Display;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum HostStatus {
    Pending,
    Up,
    Down,
}

impl FromStr for HostStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match String::from(s).to_lowercase().as_str() {
            "pending" => Ok(HostStatus::Pending),
            "up" => Ok(HostStatus::Up),
            "down" => Ok(HostStatus::Down),
            _ => panic!("Invalid HostStatus '{}'", s)
        }
    }
}

impl Default for HostStatus {
    fn default() -> Self {
        HostStatus::Pending
    }
}

impl Display for HostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostStatus::Pending => write!(f, "pending"),
            HostStatus::Up => write!(f, "up"),
            HostStatus::Down => write!(f, "down"),
        }
    }
}