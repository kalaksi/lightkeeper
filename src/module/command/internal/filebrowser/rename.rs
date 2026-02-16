/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::path::Path;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::enums;
use lightkeeper_module::command_module;

#[command_module(
    name="_internal-filebrowser-rename",
    version="0.0.1",
    description="Rename a file or directory on the remote host.",
)]
pub struct FileBrowserRename {
}

impl Module for FileBrowserRename {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserRename {}
    }
}

impl CommandModule for FileBrowserRename {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("edit-rename"),
            display_text: String::from("Rename file or directory"),
            depends_on_criticality: vec![enums::Criticality::Normal],
            action: UIAction::FollowOutput,
            tab_title: String::from("Rename"),
            parent_id: String::from("_internal-filebrowser-ls"),
            user_parameters: vec![
                frontend::UserInputField {
                    label: String::from("Path"),
                    ..Default::default()
                },
                frontend::UserInputField {
                    label: String::from("New name"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }

        let old_path = parameters.first().ok_or(LkError::other("No path specified"))?;
        let new_name = parameters.get(1).ok_or(LkError::other("No new name specified"))?;

        if old_path.is_empty() {
            return Err(LkError::other("Path is empty"));
        }
        if new_name.is_empty() {
            return Err(LkError::other("New name is empty"));
        }

        let new_path = Path::new(old_path)
            .parent()
            .ok_or(LkError::other("Invalid path"))?
            .join(new_name);
        let new_path_str = new_path.to_string_lossy();

        let mut command = ShellCommand::new();
        command.use_sudo = false;
        command.arguments(vec!["mv", "-n", old_path, &new_path_str]);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_hidden(response.message_increment.clone()))
        }
        else {
            Ok(CommandResult::new_hidden(response.message_increment.clone())
                .with_criticality(crate::enums::Criticality::Error))
        }
    }
}
