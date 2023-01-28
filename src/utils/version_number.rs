use std::str::FromStr;
use std::default::Default;
use std::fmt::Display;

use serde_derive::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct VersionNumber {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl VersionNumber {
    // Never panics or returns error. Zeroes will be used as defaults when parsing fails.
    pub fn from_string(version_string: &String) -> Self {
        let mut parts = version_string.split('.').collect::<Vec<&str>>();

        // Drop everything after dashes.
        parts = parts.iter().map(|part| part.split('-').next().unwrap_or(&"0")).collect();

        let major = parts.get(0).unwrap_or(&"0")
                         .parse::<u16>().unwrap_or_default();

        let minor = parts.get(1).unwrap_or(&"0")
                         .parse::<u16>().unwrap_or_default();

        let patch = parts.get(2).unwrap_or(&"0")
                         .parse::<u16>().unwrap_or_default();

        VersionNumber { major, minor, patch }
    }
}

impl FromStr for VersionNumber {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_string(&s.to_string()))
    }
}

impl Display for VersionNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
