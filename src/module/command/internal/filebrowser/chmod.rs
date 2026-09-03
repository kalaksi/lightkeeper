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

#[command_module(
    name="_internal-filebrowser-chmod",
    version="0.0.1",
    description="Change file permissions on the remote host.",
    uses_sudo=true,
)]
pub struct FileBrowserChmod {
}

impl Module for FileBrowserChmod {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserChmod {}
    }
}

impl CommandModule for FileBrowserChmod {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("lock"),
            display_text: String::from("Change permissions"),
            parent_id: String::from("_internal-filebrowser-ls"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }

        let path = parameters.first().ok_or(LkError::other("No path specified"))?;
        let mode = parameters.get(1).ok_or(LkError::other("No mode specified"))?;
        let owner = parameters.get(2).map(|value| value.as_str()).unwrap_or("");
        let group = parameters.get(3).map(|value| value.as_str()).unwrap_or("");
        let ownership_requested = parameters.len() >= 4;

        if path.is_empty() {
            return Err(LkError::other("Path is empty"));
        }
        if mode.is_empty() {
            return Err(LkError::other("Mode is empty"));
        }

        // GNU chmod keeps directory setuid/setgid unless a 4-digit mode that
        // clears them (0775) is written as 00775. Non-zero special bits (2775)
        // and 3-digit modes (775) are left unchanged.
        let chmod_mode = if mode.chars().all(|c| c.is_ascii_digit())
            && mode.len() == 4
            && mode.starts_with('0') {

            format!("0{mode}")
        }
        else {
            mode.clone()
        };

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);
        command.arguments(vec!["chmod", &chmod_mode, path]);

        // chown can overwrite special bits that were set with chmod, so need to guarantee order here.
        if ownership_requested {
            let owner_group = if !owner.is_empty() && !group.is_empty() {
                format!("{}:{}", owner, group)
            }
            else if !owner.is_empty() {
                owner.to_string()
            }
            else if !group.is_empty() {
                format!(":{}", group)
            }
            else {
                return Err(LkError::other("Either owner or group must be specified"));
            };

            let mut chown_command = ShellCommand::new();
            chown_command.use_sudo = host.settings.contains(&HostSetting::UseSudo);
            chown_command.arguments(vec!["chown", &owner_group, path]);

            return Ok(format!("{} && {}", chown_command.to_string(), command.to_string()));
        }

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_hidden(response.message_increment.clone()))
        }
        else {
            Ok(CommandResult::new_error(response.message_increment.clone()))
        }
    }
}
