use std::str::FromStr;

use strum_macros::{ EnumString, Display };
use serde_derive::{ Serialize, Deserialize };

use crate::utils::VersionNumber;

#[derive(Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system, i.e. Windows, Linux...
    pub os: OperatingSystem,

    /// Numeric version.
    pub os_version: VersionNumber,

    /// Flavor covers different Windows OSes and Linux distributions.
    pub os_flavor: Flavor,

    pub architecture: Architecture,
}

impl PlatformInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_unset(&self) -> bool {
        self.os == OperatingSystem::Unknown
    }

    // Version is given as str for convenience.
    pub fn is_newer_than(&self, flavor: Flavor, version: &str) -> bool {
        let parsed_version = VersionNumber::from_str(version).unwrap();

        self.os_flavor == flavor &&
        self.os_version > parsed_version
    }
}

#[derive(Clone, PartialEq, Eq, EnumString, Display, Serialize, Deserialize)]
pub enum OperatingSystem {
    Unknown,
    Windows,
    Linux,
}

impl Default for OperatingSystem {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, PartialEq, Eq, EnumString, Display, Serialize, Deserialize)]
pub enum Flavor {
    Unknown,

    // Windows:
    WindowsServer2012,
    Windows7,
    Windows10,
    Windows11,

    // Linux:
    Debian,
    Ubuntu,
    ArchLinux,
    RedHat,
}

impl Default for Flavor {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Architecture {
    Unknown,
    X86_64,
}

impl Default for Architecture {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&String> for Architecture {
    fn from(value: &String) -> Self {
        match value.to_lowercase().as_str() {
            "x86_64" => Self::X86_64,
            "x86-64" => Self::X86_64,
            "amd64" => Self::X86_64,
            _ => Self::Unknown,
        }
    }
}