/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::HashMap;

use super::ModuleSpecification;

/// What a monitor or command module does on the host (for MCP and similar access control).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleAccess {
    Read,
    Write,
    /// High blast radius / hard to undo. MCP uses `destructiveHint`; GUI treats like Write.
    Delete,
}

impl ModuleAccess {
    pub fn as_str(self) -> &'static str {
        match self {
            ModuleAccess::Read => "read",
            ModuleAccess::Write => "write",
            ModuleAccess::Delete => "delete",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "read" => Some(ModuleAccess::Read),
            "write" => Some(ModuleAccess::Write),
            "delete" => Some(ModuleAccess::Delete),
            _ => None,
        }
    }

    /// Allowed when MCP host Access is Read (read-only).
    pub fn allowed_for_read_only(self) -> bool {
        matches!(self, ModuleAccess::Read)
    }

    /// Allowed when MCP host Access is Read-write.
    pub fn allowed_for_read_write(self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct Metadata {
    pub module_spec: ModuleSpecification,
    pub description: String,
    /// Setting key and description.
    pub settings: HashMap<String, String>,
    /// Secret key and description. Same shape as settings but for values stored in e.g. keyring.
    pub secrets: HashMap<String, String>,
    /// Used with extension modules.
    /// Extension modules enrich or modify the original data and are processed after parent module.
    pub parent_module: Option<ModuleSpecification>,
    /// Stateless modules can be run in parallel.
    pub is_stateless: bool,
    /// Module uses sudo if available. True for modules that require sudo or can work with/without it.
    pub uses_sudo: bool,
    /// Required for monitors and commands. Connectors leave this as `None`.
    pub access: Option<ModuleAccess>,
}
