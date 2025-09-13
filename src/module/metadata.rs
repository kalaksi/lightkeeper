/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use super::ModuleSpecification;


#[derive(Clone, Debug)]
pub struct Metadata {
    pub module_spec: ModuleSpecification,
    pub description: String,
    /// Setting key and description.
    pub settings: HashMap<String, String>,
    /// Used with extension modules.
    /// Extension modules enrich or modify the original data and are processed after parent module.
    pub parent_module: Option<ModuleSpecification>,
    /// Stateless modules can be run in parallel.
    pub is_stateless: bool,
}
