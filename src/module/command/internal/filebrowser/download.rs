/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use regex::Regex;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::{UIAction, *};
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="_internal-filebrowser-download",
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
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("folder-download"),
            display_text: String::from("Download with rsync"),
            tab_title: String::from("Download"),
            action: UIAction::FollowOutput,
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
        let username = parameters.get(2).ok_or(LkError::other("Remote user is required"))?;

        // If empty username, don't set user at all (defaults to current user)..
        let remote_spec = match username.is_empty() {
            true => format!("{}:{}", host.get_address(), remote_path),
            false => format!("{}@{}:{}", username, host.get_address(), remote_path),
        };

        if remote_path.len() == 0 {
            return Err(LkError::other("Remote path is empty"));
        }
        if local_path.len() == 0 {
            return Err(LkError::other("Local path is empty"));
        }

        let mut command = ShellCommand::new();
        command.use_sudo = false;
        command.arguments(vec![
            // "sudo",
            // "--preserve-env=SSH_AUTH_SOCK",
            // Try to keep output format more stable.
            "env", "LANG=C", "LC_ALL=C",
            "rsync",
            "-avz",
            "--info=progress2",
            "--stats",
            // "--rsync-path=sudo rsync",
            &remote_spec,
            local_path,
        ]);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            let progress = parse_rsync_progress(&response.message);
            Ok(CommandResult::new_partial(response.message_increment.clone(), progress))
        }
        else {
            Ok(process_rsync_final_response(response, "Download"))
        }
    }
}

/// Parses rsync --info=progress2 output and returns the last percentage (0-100).
pub fn parse_rsync_progress(message: &str) -> u8 {
    Regex::new(r"(\d+)%")
        .unwrap()
        .find_iter(message)
        .filter_map(|m| m.as_str().trim_end_matches('%').parse::<u8>().ok())
        .filter(|&p| p <= 100)
        .last()
        .unwrap_or(10)
}

/// Parses rsync summary. Returns (skipped_count, transferred_count) when some files were skipped.
pub fn parse_rsync_skipped_count(message: &str) -> Option<(u32, u32)> {
    let transferred_re =
        Regex::new(r"Number of (?:regular )?files transferred:\s*(\d+)").ok()?;
    let transferred: u32 = transferred_re.captures(message)?.get(1)?.as_str().parse().ok()?;
    let files_re =
        Regex::new(r"Number of files:\s*(\d+)(?:\s*\(reg:\s*(\d+)(?:,\s*dir:\s*\d+)?\))?").ok()?;
    let files_caps = files_re.captures(message)?;
    let total_regular: u32 = files_caps
        .get(2)
        .and_then(|m| m.as_str().parse().ok())
        .or_else(|| files_caps.get(1).and_then(|m| m.as_str().parse().ok()))?;
    let skipped = total_regular.saturating_sub(transferred);
    if skipped > 0 {
        Some((skipped, transferred))
    }
    else {
        None
    }
}

/// Builds CommandResult for a non-partial rsync response. Use for copy, download, upload.
pub fn process_rsync_final_response(
    response: &ResponseMessage,
    operation: &str,
) -> CommandResult {
    if response.return_code != 0 {
        return CommandResult::new_error(response.message.clone());
    }
    if let Some((skipped, transferred)) = parse_rsync_skipped_count(&response.message) {
        let msg = if transferred == 0 {
            "All files were skipped (maybe already existed at destination).".to_string()
        }
        else {
            format!(
                "{} completed. {} files were skipped.",
                operation, skipped
            )
        };
        return CommandResult::new_info(msg);
    }
    CommandResult::new_hidden(response.message_increment.clone())
}
