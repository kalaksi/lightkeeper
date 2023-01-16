use strum_macros::{ EnumString, Display };

#[derive(Default, Clone)]
pub struct PlatformInfo {
    /// Operating system, i.e. Windows, Linux...
    pub os: OperatingSystem,

    /// Numeric version.
    pub os_version: String,

    /// Flavor covers different Windows OSes and Linux distributions.
    pub os_flavor: Flavor,
}

#[derive(Clone, PartialEq, Eq, EnumString, Display)]
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

#[derive(Clone, PartialEq, Eq, EnumString, Display)]
pub enum Flavor {
    Unknown,

    // Windows:
    Server2012,
    Windows7,
    Windows10,
    Windows11,

    // Linux:
    Debian,
    Ubuntu,
    ArchLinux,
    RHEL,
}

impl Default for Flavor {
    fn default() -> Self {
        Self::Unknown
    }
}