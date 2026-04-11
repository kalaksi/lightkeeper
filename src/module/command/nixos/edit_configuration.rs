/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::*;
use crate::module::command::*;
use lightkeeper_module::command_module;

#[command_module(
    name="nixos-edit-configuration",
    version="0.0.1",
    description="Launches an editor for editing configuration.nix.",
)]
pub struct EditConfiguration;

impl Module for EditConfiguration {
    fn new(_settings: &HashMap<String, String>) -> EditConfiguration {
        EditConfiguration
    }
}

impl CommandModule for EditConfiguration {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("nixos"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("story-editor"),
            display_text: String::from("Edit configuration.nix"),
            action: UIAction::TextEditor,
            tab_title: String::from("%s"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        Ok(String::from("/etc/nixos/configuration.nix"))
    }
}
