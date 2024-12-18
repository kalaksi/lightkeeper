use std::default::Default;
use std::fmt::Display;
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct VersionNumber {
    major: u16,
    // In comparisons, None will always be considered less than some number, so in this context it's effectively 0.
    minor: Option<u16>,
    patch: Option<u16>,
}

impl VersionNumber {
    // Never panics or returns error. Zeroes will be used as defaults when parsing fails.
    pub fn from_string(version_string: &str) -> Self {
        let mut parts = version_string.split('.').collect::<Vec<&str>>();

        // Drop everything after dashes.
        parts = parts.iter().map(|part| part.split('-').next().unwrap_or("0")).collect();

        let major = parts.first().unwrap_or(&"0").parse::<u16>().unwrap_or_default();

        let minor = match parts.get(1) {
            Some(minor) => minor.parse::<u16>().ok(),
            None => None,
        };

        let patch = match parts.get(2) {
            Some(patch) => patch.parse::<u16>().ok(),
            None => None,
        };

        VersionNumber { major, minor, patch }
    }
}

impl FromStr for VersionNumber {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_string(s))
    }
}

impl Display for VersionNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.minor {
            Some(minor) => match self.patch {
                Some(patch) => write!(f, "{}.{}.{}", self.major, minor, patch),
                None => write!(f, "{}.{}", self.major, minor),
            },
            None => write!(f, "{}", self.major),
        }
    }
}
