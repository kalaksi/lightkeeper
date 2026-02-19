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
use crate::enums;
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
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("lock"),
            display_text: String::from("Change permissions"),
            depends_on_criticality: vec![enums::Criticality::Normal],
            action: UIAction::FollowOutput,
            tab_title: String::from("Chmod"),
            parent_id: String::from("_internal-filebrowser-ls"),
            user_parameters: vec![
                frontend::UserInputField {
                    label: String::from("Path"),
                    ..Default::default()
                },
                frontend::UserInputField {
                    label: String::from("Mode (octal)"),
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

        let path = parameters.first().ok_or(LkError::other("No path specified"))?;
        let mode = parameters.get(1).ok_or(LkError::other("No mode specified"))?;

        if path.is_empty() {
            return Err(LkError::other("Path is empty"));
        }
        if mode.is_empty() {
            return Err(LkError::other("Mode is empty"));
        }

        let mut command = ShellCommand::new();
        command.use_sudo = true;
        command.arguments(vec!["chmod", mode, path]);

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
