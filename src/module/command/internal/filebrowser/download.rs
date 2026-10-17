/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::path::PathBuf;
use regex::Regex;
use crate::error::LkError;
use crate::file_handler;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::{UIAction, *};
use crate::utils::ShellCommand;
use serde::{Deserialize, Serialize};
use lightkeeper_module::command_module;

#[command_module(
    name="_internal-filebrowser-download",
    version="0.0.1",
    description="Download remote files with rsync.",
    uses_sudo=true,
)]
pub struct FileBrowserDownload {
    ssh: RsyncSsh,
}

impl Module for FileBrowserDownload {
    fn new(settings: &HashMap<String, String>) -> Self {
        FileBrowserDownload {
            ssh: RsyncSsh::new(settings),
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

        if remote_path.is_empty() {
            return Err(LkError::other("Remote path is empty"));
        }
        if local_path.is_empty() {
            return Err(LkError::other("Local path is empty"));
        }

        let remote_spec = self.ssh.remote_spec(&host.get_address(), remote_path, false)?;

        let mut command = ShellCommand::new();
        command.use_sudo = false;
        command.arguments(vec!["env", "LANG=C", "LC_ALL=C", "rsync", "-avz", "--info=progress2", "--stats", RSYNC_OUT_FORMAT]);
        command.argument("-e");
        command.argument(rsync_remote_shell(&self.ssh.ssh_arguments()?)?);
        if host.should_use_sudo() {
            command.argument("--rsync-path=sudo rsync");
        }
        command.arguments(vec![remote_spec, local_path.clone()]);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            Ok(process_rsync_partial_response(response, "Downloading"))
        }
        else {
            Ok(process_rsync_final_response(response, "Download"))
        }
    }
}

/// SSH connector settings needed for rsync's remote shell.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RsyncSsh {
    pub username: String,
    pub port: u16,
    pub private_key_path: Option<String>,
    pub verify_host_key: bool,
    pub custom_known_hosts_path: Option<PathBuf>,
}

impl RsyncSsh {
    pub fn new(ssh_settings: &HashMap<String, String>) -> Self {
        RsyncSsh {
            username: ssh_settings.get("username").cloned().unwrap_or_default(),
            port: ssh_settings.get("port").and_then(|v| v.parse().ok()).unwrap_or(22),
            private_key_path: ssh_settings.get("private_key_path").cloned(),
            verify_host_key: ssh_settings.get("verify_host_key")
                .and_then(|v| v.parse().ok()).unwrap_or(true),
            custom_known_hosts_path: ssh_settings.get("custom_known_hosts_path")
                .map(PathBuf::from),
        }
    }

    /// Builds the remote `user@host:path` side of an rsync transfer.
    /// Address and username are validated so the spec can't become an rsync option or a local path.
    pub fn remote_spec(&self, address: &str, remote_path: &str, for_upload: bool) -> Result<String, LkError> {
        let is_invalid = |value: &str| {
            value.starts_with('-') ||
                value.contains(|c: char| matches!(c, ':' | '@' | '/') || c.is_whitespace() || c.is_control())
        };
        if address.is_empty() || is_invalid(address) || is_invalid(&self.username) {
            return Err(LkError::other("Invalid host address or username"));
        }

        let path = if for_upload {
            if remote_path.ends_with('/') {
                remote_path.to_string()
            }
            else {
                format!("{}/", remote_path)
            }
        }
        else {
            // Directory paths from the file browser end with '/'.
            // Strip it so rsync copies whole directories instead of just their contents.
            let trimmed = remote_path.trim_end_matches('/');
            if trimmed.is_empty() { remote_path.to_string() } else { trimmed.to_string() }
        };

        if self.username.is_empty() {
            Ok(format!("{}:{}", address, path))
        }
        else {
            Ok(format!("{}@{}:{}", self.username, address, path))
        }
    }

    /// Custom known hosts path or the default one, which is created if missing.
    pub fn known_hosts_path(&self) -> Result<PathBuf, LkError> {
        match &self.custom_known_hosts_path {
            Some(path) => Ok(path.clone()),
            None => {
                let path = file_handler::get_config_dir().join("known_hosts");
                if !path.exists() {
                    std::fs::File::create(&path)
                        .map_err(|error| LkError::other(format!("{}", error)))?;
                }
                Ok(path)
            }
        }
    }

    /// Unquoted ssh arguments for rsync's remote shell.
    pub fn ssh_arguments(&self) -> Result<Vec<String>, LkError> {
        let mut arguments = vec![String::from("ssh"), String::from("-p"), self.port.to_string()];
        if let Some(key_path) = &self.private_key_path {
            arguments.extend([String::from("-i"), key_path.clone()]);
        }
        if self.verify_host_key {
            arguments.extend([
                String::from("-o"),
                format!("UserKnownHostsFile={}", self.known_hosts_path()?.to_string_lossy()),
                String::from("-o"),
                String::from("GlobalKnownHostsFile=/dev/null"),
            ]);
        }
        else {
            arguments.extend([
                String::from("-o"),
                String::from("StrictHostKeyChecking=no"),
                String::from("-o"),
                String::from("UserKnownHostsFile=/dev/null"),
            ]);
        }
        Ok(arguments)
    }
}

/// Joins arguments for rsync's `-e`. rsync splits the remote shell command itself and only supports quotes
/// (no escaping), so arguments containing double quotes are rejected.
pub fn rsync_remote_shell(arguments: &[String]) -> Result<String, LkError> {
    if arguments.iter().any(|argument| argument.contains('"')) {
        return Err(LkError::other("Remote shell arguments can't contain double quotes"));
    }
    Ok(arguments.iter().map(|argument| format!("\"{}\"", argument)).collect::<Vec<_>>().join(" "))
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

/// Marker prepended to each transferred path via rsync's --out-format, so we can reliably
/// pick out the current file name from output that is otherwise interleaved with progress
/// lines and summary stats. Keep in sync with RSYNC_OUT_FORMAT.
pub const RSYNC_FILE_MARKER: &str = "@@LKFILE@@";
/// rsync --out-format value that prepends RSYNC_FILE_MARKER to each transferred path.
pub const RSYNC_OUT_FORMAT: &str = "--out-format=@@LKFILE@@%f";

/// Returns the base name of the file currently being transferred, parsed from the marker
/// emitted by RSYNC_OUT_FORMAT. Returns None when no file name is present (e.g. progress-only
/// or summary output), so callers can keep showing the previously parsed file.
pub fn parse_rsync_current_file(message: &str) -> Option<String> {
    let start = message.rfind(RSYNC_FILE_MARKER)? + RSYNC_FILE_MARKER.len();
    // Take everything after the last marker up to the next line boundary.
    let rest = message[start..].split(['\r', '\n']).next().unwrap_or("");
    // A progress line can be glued onto the same line, e.g. "dir/file   1,234 100% ...".
    // Cut it off at the start of the progress numbers.
    let progress_re = Regex::new(r"\s+[\d,]+\s+\d+%").unwrap();
    let name = match progress_re.find(rest) {
        Some(m) => &rest[..m.start()],
        None => rest,
    }
    .trim();

    // Skip directory entries (rsync lists e.g. "dir/." and "dir/").
    if name.is_empty() || name == "." || name.ends_with('/') || name.ends_with("/.") {
        return None;
    }
    let base = name.rsplit('/').next().unwrap_or(name);
    if base.is_empty() {
        None
    }
    else {
        Some(base.to_string())
    }
}

pub fn process_rsync_partial_response(response: &ResponseMessage, operation: &str) -> CommandResult {
    let progress = parse_rsync_progress(&response.message);
    let message = match parse_rsync_current_file(&response.message) {
        Some(file) => format!("{} {}", operation, file),
        None => String::new(),
    };
    CommandResult::new_partial(message, progress)
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
pub fn process_rsync_final_response(response: &ResponseMessage, operation: &str) -> CommandResult {
    // User cancelled the transfer, so don't surface it as an error notification.
    if response.return_code == 20 {
        return CommandResult::new_hidden(format!("{} cancelled", operation));
    }
    if response.return_code != 0 {
        let text = if response.message.is_empty() {
            format!("Command failed with exit code {}.", response.return_code)
        }
        else {
            response.message.clone()
        };
        return CommandResult::new_error(text);
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
    CommandResult::new_hidden(response.message.clone())
}
