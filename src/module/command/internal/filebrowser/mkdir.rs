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
use lightkeeper_module::command_module;

#[command_module(
    name="_internal-filebrowser-mkdir",
    version="0.0.1",
    description="Create a directory on the remote host.",
    uses_sudo=true,
)]
pub struct FileBrowserMkdir {
}

impl Module for FileBrowserMkdir {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserMkdir {}
    }
}

impl CommandModule for FileBrowserMkdir {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("folder-new"),
            display_text: String::from("Create directory"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }

        let parent_dir = parameters.first().ok_or(LkError::other("No parent directory specified"))?;
        let folder_name = parameters.get(1).ok_or(LkError::other("No folder name specified"))?;

        if parent_dir.is_empty() {
            return Err(LkError::other("Parent directory is empty"));
        }
        if folder_name.is_empty() {
            return Err(LkError::other("Folder name is empty"));
        }

        let new_path = Path::new(parent_dir).join(folder_name);
        let new_path_str = new_path.to_string_lossy();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);
        command.arguments(vec!["mkdir", "--", &new_path_str]);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_hidden(response.message_increment.clone()))
        }
        else {
            let msg = if response.message.is_empty() {
                response.message_increment.clone()
            }
            else {
                response.message.clone()
            };
            Ok(CommandResult::new_error(if msg.is_empty() { "Failed to create directory" } else { &msg }))
        }
    }
}
