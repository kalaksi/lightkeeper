/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::path::PathBuf;
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
    username: String,
    port: u16,
    private_key_path: Option<String>,
    verify_host_key: bool,
    custom_known_hosts_path: Option<PathBuf>,
}

impl Module for FileBrowserUpload {
    fn new(settings: &HashMap<String, String>) -> Self {
        FileBrowserUpload {
            username: settings.get("username").cloned().unwrap_or_default(),
            port: settings.get("port").and_then(|v| v.parse().ok()).unwrap_or(22),
            private_key_path: settings.get("private_key_path").cloned(),
            verify_host_key: settings.get("verify_host_key")
                .and_then(|v| v.parse().ok()).unwrap_or(true),
            custom_known_hosts_path: settings.get("custom_known_hosts_path")
                .map(PathBuf::from),
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

        let remote_dir = if remote_path.ends_with('/') {
            remote_path.clone()
        } else {
            format!("{}/", remote_path)
        };
        let remote_spec = match self.username.is_empty() {
            true => format!("{}:{}", host.get_address(), remote_dir),
            false => format!("{}@{}:{}", self.username, host.get_address(), remote_dir),
        };

        if local_path.is_empty() {
            return Err(LkError::other("Local path is empty"));
        }
        if remote_path.is_empty() {
            return Err(LkError::other("Remote path is empty"));
        }

        let mut command = ShellCommand::new();
        command.use_sudo = false;
        command.arguments(vec!["env", "LANG=C", "LC_ALL=C", "rsync", "-avz", "--info=progress2", "--stats"]);
        command.argument("-e");
        command.argument(download::build_rsync_ssh_command(
            self.port,
            self.private_key_path.as_deref(),
            self.verify_host_key,
            self.custom_known_hosts_path.as_deref(),
        )?);

        if host.settings.contains(&HostSetting::UseSudo) {
            command.argument("--rsync-path=sudo rsync");
        }
        command.arguments(vec![local_path.clone(), remote_spec]);

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
