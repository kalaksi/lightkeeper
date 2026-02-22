/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;
use serde_json::json;

use crate::module::command::internal::filebrowser::ls::parse_ls_output;

#[command_module(
    name="storage-filebrowser",
    version="0.0.1",
    description="Open file browser at a mount point.",
)]
pub struct FileBrowser {
}

impl Module for FileBrowser {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowser {}
    }
}

impl CommandModule for FileBrowser {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("storage"),
            parent_id: String::from("filesystem"),
            multivalue_level: 1,
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("view-list-tree"),
            display_text: String::from("Open file browser"),
            action: UIAction::FileBrowser,
            tab_title: String::from("File browser"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = false;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            let path = parameters.first().ok_or(LkError::other("No path specified"))?.as_str();
            command.arguments(vec![
                "ls", "-lAL", "--group-directories-first", "--color=never", "--time-style=long-iso", path
            ]);
        }
        else {
            return Err(LkError::unsupported_platform());
        }

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        let entries = parse_ls_output(&response.message)?;
        let json_result = json!({ "entries": entries });
        Ok(CommandResult::new_hidden(json_result.to_string()))
    }
}
