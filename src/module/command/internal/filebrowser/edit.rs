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
    name="_internal-filebrowser-edit",
    version="0.0.1",
    description="Edit a remote file.",
)]
pub struct FileBrowserEdit {
}

impl Module for FileBrowserEdit {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserEdit {
        }
    }
}

impl CommandModule for FileBrowserEdit {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("story-editor"),
            display_text: String::from("Edit file"),
            action: UIAction::TextEditor,
            tab_title: String::from("Editor"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, _host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let path = parameters.first().ok_or(LkError::other("No path specified"))?.clone();
        Ok(path)
    }
}
