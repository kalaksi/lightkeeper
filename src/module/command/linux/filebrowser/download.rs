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
    name="linux-filebrowser-download",
    version="0.0.1",
    description="Download remote files with rsync.",
)]
pub struct FileBrowserDownload {
}

impl Module for FileBrowserDownload {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserDownload {
        }
    }
}

impl CommandModule for FileBrowserDownload {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("local-command", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("folder-download"),
            display_text: String::from("Download"),
            depends_on_criticality: vec![enums::Criticality::Normal],
            action: UIAction::FollowOutput,
            tab_title: String::from("Download"),
            parent_id: String::from("linux-filebrowser-ls"),
            user_parameters: vec![
                frontend::UserInputField {
                    label: String::from("Remote path"),
                    ..Default::default()
                },
                frontend::UserInputField {
                    label: String::from("Local path"),
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

        let remote_path = parameters.first().ok_or(LkError::other("No remote path specified"))?;
        let local_path = parameters.get(1).ok_or(LkError::other("No local path specified"))?;
        let username = parameters.get(2).map(String::as_str).unwrap_or("root");
        let remote_spec = format!("{}@{}:{}", username, host.get_address(), remote_path);

        let mut command = ShellCommand::new();
        command.use_sudo = false;
        command.arguments(vec![
            "sudo",
            "--preserve-env=SSH_AUTH_SOCK",
            "rsync",
            "-avz",
            "--info=progress2",
            "--rsync-path=sudo rsync",
            &remote_spec,
            local_path,
        ]);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_info(response.message.clone()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}
