/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::utils::{string_manipulation, VersionNumber};

#[derive(Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system, i.e. Windows, Linux...
    pub os: OperatingSystem,

    /// Numeric version.
    pub os_version: VersionNumber,

    /// Flavor covers different Windows OSes and Linux distributions.
    pub os_flavor: Flavor,

    /// From VARIANT_ID in os-release when present (e.g. coreos for Fedora CoreOS).
    #[serde(default)]
    pub os_variant_id: String,

    pub architecture: Architecture,
}

impl PlatformInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn linux(flavor: Flavor, version: &str) -> Self {
        let Ok(parsed_version) = VersionNumber::from_str(version)
        else {
            panic!("Invalid version {}", version);
        };

        PlatformInfo {
            os: OperatingSystem::Linux,
            architecture: Architecture::X86_64,
            os_flavor: flavor,
            os_version: parsed_version,
            os_variant_id: String::new(),
        }
    }

    /// Build platform info from the same probes as `_internal-platform-info-ssh`
    /// (`cat /etc/os-release` and `uname -m`).
    pub fn from_linux_probe(os_release: &str, uname_machine: &str) -> Self {
        let (os_flavor, os_version, os_variant_id) = parse_os_release(os_release);
        PlatformInfo {
            os: OperatingSystem::Linux,
            os_version,
            os_flavor,
            os_variant_id,
            architecture: Architecture::from(&uname_machine.trim()),
        }
    }

    pub fn is_set(&self) -> bool {
        self.os != OperatingSystem::Unknown
    }

    // Version is given as str for convenience.
    pub fn is_same_or_greater(&self, flavor: Flavor, version: &str) -> bool {
        let Ok(parsed_version) = VersionNumber::from_str(version)
        else {
            panic!("Invalid version {}", version);
        };

        self.os_flavor == flavor && self.os_version >= parsed_version
    }

    pub fn is_variant(&self, flavor: Flavor, variant_id: &str) -> bool {
        self.os_flavor == flavor && self.os_variant_id == variant_id
    }
}

/// Parses `/etc/os-release` contents into flavor, version, and variant id.
pub fn parse_os_release(message: &str) -> (Flavor, VersionNumber, String) {
    let mut flavor = Flavor::default();
    let mut version = VersionNumber::default();
    let mut variant_id = String::new();

    for line in message.lines() {
        let mut parts = line.split('=');
        let key = parts.next().unwrap_or_default();
        let value = string_manipulation::remove_quotes(&parts.next().unwrap_or_default());

        match key {
            "ID" => {
                match value.as_str() {
                    "debian" => flavor = Flavor::Debian,
                    "centos" => flavor = Flavor::CentOS,
                    "ubuntu" => flavor = Flavor::Ubuntu,
                    "nixos" => flavor = Flavor::NixOS,
                    "arch" => flavor = Flavor::ArchLinux,
                    "fedora" => flavor = Flavor::Fedora,
                    "opensuse" => flavor = Flavor::OpenSUSE,
                    "alpine" => flavor = Flavor::Alpine,
                    _ => ()
                }
            },
            "VERSION_ID" => version = VersionNumber::from_string(&value.to_string()),
            "VARIANT_ID" => variant_id = value,
            _ => ()
        }
    }

    (flavor, version, variant_id)
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
    CentOS,
    NixOS,
    Fedora,
    OpenSUSE,
    Alpine,
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
    Arm64,
    Arm,
}

impl Default for Architecture {
    fn default() -> Self {
        Self::Unknown
    }
}

impl<Stringable: ToString> From<&Stringable> for Architecture {
    fn from(value: &Stringable) -> Self {
        match value.to_string().to_lowercase().as_str() {
            "x86_64" => Self::X86_64,
            "x86-64" => Self::X86_64,
            "amd64" => Self::X86_64,
            "aarch64" => Self::Arm64,
            "arm64" => Self::Arm64,
            "arm" => Self::Arm,
            _ => Self::Unknown,
        }
    }
}
