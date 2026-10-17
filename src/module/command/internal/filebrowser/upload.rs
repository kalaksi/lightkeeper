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
    uses_sudo=true,
)]
pub struct FileBrowserUpload {
    ssh: download::RsyncSsh,
}

impl Module for FileBrowserUpload {
    fn new(settings: &HashMap<String, String>) -> Self {
        FileBrowserUpload {
            ssh: download::RsyncSsh::new(settings),
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

        if local_path.is_empty() {
            return Err(LkError::other("Local path is empty"));
        }
        if remote_path.is_empty() {
            return Err(LkError::other("Remote path is empty"));
        }

        let remote_spec = self.ssh.remote_spec(&host.get_address(), remote_path, true)?;

        let mut command = ShellCommand::new();
        command.use_sudo = false;
        command.arguments(vec!["env", "LANG=C", "LC_ALL=C", "rsync", "-avz", "--info=progress2", "--stats", download::RSYNC_OUT_FORMAT]);
        command.argument("-e");
        command.argument(download::rsync_remote_shell(&self.ssh.ssh_arguments()?)?);

        if host.should_use_sudo() {
            command.argument("--rsync-path=sudo rsync");
        }
        command.arguments(vec![local_path.clone(), remote_spec]);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            Ok(download::process_rsync_partial_response(response, "Uploading"))
        }
        else {
            Ok(download::process_rsync_final_response(response, "Upload"))
        }
    }
}
