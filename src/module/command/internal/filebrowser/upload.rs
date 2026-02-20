/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use crate::error::LkError;
use super::download;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::{UIAction, *};
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="_internal-filebrowser-upload",
    version="0.0.1",
    description="Upload local files with rsync.",
)]
pub struct FileBrowserUpload {
}

impl Module for FileBrowserUpload {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserUpload {
        }
    }
}

impl CommandModule for FileBrowserUpload {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("local-command", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("folder-upload"),
            display_text: String::from("Upload with rsync"),
            tab_title: String::from("Upload"),
            parent_id: String::from("_internal-filebrowser-ls"),
            action: UIAction::FollowOutput,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }

        let local_path = parameters.first().ok_or(LkError::other("No local path specified"))?;
        let remote_path = parameters.get(1).ok_or(LkError::other("No remote path specified"))?;
        let username = parameters.get(2).ok_or(LkError::other("Remote user is required"))?;

        let remote_dir = if remote_path.ends_with('/') {
            remote_path.clone()
        } else {
            format!("{}/", remote_path)
        };
        let remote_spec = match username.is_empty() {
            true => format!("{}:{}", host.get_address(), remote_dir),
            false => format!("{}@{}:{}", username, host.get_address(), remote_dir),
        };

        if local_path.is_empty() {
            return Err(LkError::other("Local path is empty"));
        }
        if remote_path.is_empty() {
            return Err(LkError::other("Remote path is empty"));
        }

        let mut command = ShellCommand::new();
        command.use_sudo = false;
        command.arguments(vec![
            // Try to keep output format more stable.
            "env", "LANG=C", "LC_ALL=C",
            "rsync",
            "-avz",
            "--info=progress2",
            "--stats",
            local_path,
            &remote_spec,
        ]);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            let progress = download::parse_rsync_progress(&response.message);
            Ok(CommandResult::new_partial(response.message_increment.clone(), progress))
        }
        else {
            Ok(download::process_rsync_final_response(response, "Upload"))
        }
    }
}
