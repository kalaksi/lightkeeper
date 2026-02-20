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
use crate::module::command::UIAction;
use lightkeeper_module::command_module;
use serde_json::json;

use super::ls;

#[command_module(
    name="_internal-filebrowser-ls-links",
    version="0.0.1",
    description="List files in a directory (with symlinks).",
)]
pub struct FileBrowserLsLinks {
}

impl Module for FileBrowserLsLinks {
    fn new(_settings: &std::collections::HashMap<String, String>) -> Self {
        FileBrowserLsLinks {}
    }
}

impl CommandModule for FileBrowserLsLinks {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("view-list-tree"),
            display_text: String::from("List files (with symlinks)"),
            action: UIAction::FileBrowser,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = false;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            let path = parameters.first().ok_or(LkError::other("No path specified"))?.as_str();
            command.arguments(vec!["ls", "-lA", "--group-directories-first", "--color=never", "--time-style=long-iso", path]);
        }
        else {
            return Err(LkError::unsupported_platform());
        }

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        let entries = ls::parse_ls_output(&response.message)?;
        let json_result = json!({
            "entries": entries
        });
        Ok(CommandResult::new_hidden(json_result.to_string()))
    }
}