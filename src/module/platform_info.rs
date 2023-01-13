
#[derive(Default, Clone)]
pub struct PlatformInfo {
    /// Operating system, i.e. Windows, Linux...
    pub os: OperatingSystem,

    /// Numeric version.
    pub os_version: String,

    /// Flavor covers different Windows OSes and Linux distributions.
    pub os_flavor: Flavor,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub enum Flavor {
    Unknown,

    // Windows:
    Server2012,
    Windows7,
    Windows10,
    Windows11,

    // Linux:
    Debian,
    ArchLinux,
}

impl Default for Flavor {
    fn default() -> Self {
        Self::Unknown
    }
}