/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use crate::module::metadata::Metadata;
use crate::module::ModuleSpecification;

pub trait Module {
    fn new(settings: &HashMap<String, String>) -> Self where Self: Sized;
}

pub trait MetadataSupport {
    fn get_metadata() -> Metadata where Self: Sized;
    fn get_metadata_self(&self) -> Metadata;
    fn get_module_spec(&self) -> ModuleSpecification;

    /// IDs prefixed with _ are reserved for internal use.
    fn is_internal(&self) -> bool {
        self.get_module_spec().is_internal()
    }
}