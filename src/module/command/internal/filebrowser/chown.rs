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
    name="_internal-filebrowser-chown",
    version="0.0.1",
    description="Change file ownership on the remote host.",
    uses_sudo=true,
)]
pub struct FileBrowserChown {
}

impl Module for FileBrowserChown {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserChown {}
    }
}

impl CommandModule for FileBrowserChown {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("user"),
            display_text: String::from("Change ownership"),
            parent_id: String::from("_internal-filebrowser-ls"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }

        let path = parameters.first().ok_or(LkError::other("No path specified"))?;
        let owner = parameters.get(1).map(|s| s.as_str()).unwrap_or("");
        let group = parameters.get(2).map(|s| s.as_str()).unwrap_or("");

        if path.is_empty() {
            return Err(LkError::other("Path is empty"));
        }

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

        let mut command = ShellCommand::new();
        command.use_sudo = true;
        command.arguments(vec!["chown", &owner_group, path]);

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
