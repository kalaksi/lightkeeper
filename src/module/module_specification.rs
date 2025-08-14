/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fmt::Display;

use strum_macros::{Display, EnumString};

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ModuleSpecification {
    pub id: String,
    pub version: String,
    pub module_type: ModuleType,
}

impl ModuleSpecification {
    pub fn new(id: &str, version: &str, module_type: ModuleType) -> Self {
        if id.chars().any(char::is_whitespace) {
            panic!("No whitespace allowed in module ID.");
        }

        ModuleSpecification {
            id: id.to_string(),
            version: version.to_string(),
            module_type: module_type,
        }
    }

    pub fn connector(id: &str, version: &str) -> Self {
        ModuleSpecification::new(id, version, ModuleType::Connector)
    }

    pub fn command(id: &str, version: &str) -> Self {
        ModuleSpecification::new(id, version, ModuleType::Command)
    }

    pub fn monitor(id: &str, version: &str) -> Self {
        ModuleSpecification::new(id, version, ModuleType::Monitor)
    }

    pub fn latest_version(&self) -> bool {
        self.version == "latest"
    }

    /// IDs prefixed with _ are reserved for internal use.
    pub fn is_internal(&self) -> bool {
        self.id.starts_with("_")
    }
}

impl Display for ModuleSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.version)
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Display, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ModuleType {
    #[default]
    Unknown,
    Command,
    Monitor,
    Connector,
}
