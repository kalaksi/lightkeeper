/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::{HashMap, HashSet};
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::sh_single_quoted;
use lightkeeper_module::command_module;
use serde_json::{json, Value};

pub const RESOLVED_DIRECTORIES_MARKER: &str = "Resolved directories:";

#[command_module(
    name="_internal-filebrowser-ls",
    version="0.0.1",
    description="List files and directories.",
    uses_sudo=true,
)]
pub struct FileBrowserLs {
}

impl Module for FileBrowserLs {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserLs {
        }
    }
}

impl CommandModule for FileBrowserLs {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("view-list-tree"),
            display_text: String::from("Open file browser"),
            action: UIAction::FileBrowser,
            tab_title: String::from("File browser"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }
        let path = parameters.first().ok_or(LkError::other("No path specified"))?.as_str();
        Ok(linux_ls_connector_message(path, host.should_use_sudo()))
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        // Prefer a parsed listing over treating a non-zero exit as a hard failure.
        match parse_ls_output(&response.message) {
            Ok(entries) => {
                let json_result = json!({
                    "entries": entries
                });
                Ok(CommandResult::new_hidden(json_result.to_string()))
            },
            Err(_) if response.return_code != 0 => {
                let msg = if response.message.trim().is_empty() {
                    format!("Failed to list directory (exit code {})", response.return_code)
                }
                else {
                    response.message.clone()
                };
                Ok(CommandResult::new_error(msg))
            },
            Err(message) => Err(message),
        }
    }
}

/// Shared listing used by the file browser: `ls` without `-L` (keeps `name -> target`), then
/// names of symlinks that resolve to directories for the left-hand tree.
pub fn linux_ls_connector_message(path: &str, use_sudo: bool) -> String {
    let sudo = if use_sudo { "sudo " } else { "" };
    let path = sh_single_quoted(path);
    // --full-time works on both GNU coreutils and BusyBox; --time-style does not.
    format!(
        concat!(
            "{sudo}ls -lA --group-directories-first --color=never --full-time -- {path} && ",
            "echo '{marker}' && ",
            "for f in {path}/.* {path}/*; do ",
            "{sudo}test -h \"$f\" && {sudo}test -d \"$f\" && printf '%s\\n' \"${{f##*/}}\"; ",
            "done"
        ),
        sudo = sudo,
        path = path,
        marker = RESOLVED_DIRECTORIES_MARKER,
    )
}

pub fn parse_ls_output(output: &str) -> Result<Vec<Value>, String> {
    let mut entries = Vec::new();
    let all_lines: Vec<&str> = output.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    let mut lines = all_lines.iter().copied();

    // Valid `ls -l` directory listings always include "total N". Diagnostics can appear before
    // that line when stderr is merged into the response.
    let mut saw_content = false;
    loop {
        match lines.next() {
            Some(line) if line.starts_with("total ") => break,
            Some(_) => {
                saw_content = true;
            },
            None if saw_content => return Err(output.trim().to_string()),
            None => return Ok(entries),
        }
    }

    let mut saw_invalid_row = false;
    let mut resolved_directories = HashSet::new();
    while let Some(line) = lines.next() {
        if line == RESOLVED_DIRECTORIES_MARKER {
            for name in lines {
                if name != "." && name != ".." {
                    resolved_directories.insert(name.to_string());
                }
            }
            break;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        // permissions links owner group size date time [timezone] name...
        if parts.len() < 8 {
            log::warn!("Invalid line: {}", line);
            saw_invalid_row = true;
            continue;
        }

        let permissions = parts[0];
        // Skip diagnostics and other non-row text instead of failing the whole listing.
        if !is_valid_ls_permissions(permissions) {
            log::warn!("Invalid line: {}", line);
            saw_invalid_row = true;
            continue;
        }

        let links = parts[1];
        let owner = parts[2];
        let group = parts[3];
        let size = parts[4];
        let date = parts[5];
        let time = parts[6];

        // --full-time adds a timezone after time (BusyBox and GNU).
        let timezone = parts[7].as_bytes();
        let name_start = if timezone.len() >= 5
            && matches!(timezone[0], b'+' | b'-')
            && timezone[1..].iter().all(|&byte| byte.is_ascii_digit() || byte == b':')
        {
            8
        }
        else {
            7
        };
        if parts.len() <= name_start {
            log::warn!("Invalid line: {}", line);
            continue;
        }

        // Everything from name_start onwards is the name (and optional ` -> target`).
        let name_and_target = parts[name_start..].join(" ");
        let (name, link_target) = match name_and_target.split_once(" -> ") {
            Some((name, target)) => (name.to_string(), Some(target.to_string())),
            None => (name_and_target, None),
        };

        // Determine file type from permissions first character
        let file_type = match permissions.chars().next() {
            Some('d') => "d",  // directory
            Some('l') => "l",  // symbolic link
            Some('c') => "c",  // character device
            Some('b') => "b",  // block device
            Some('p') => "p",  // named pipe (FIFO)
            Some('s') => "s",  // socket
            Some('-') => "f",  // regular file
            _ => "f",          // default to file
        };

        // Directory size from `ls` is only metadata; show 0 until `du` is used.
        let size = if file_type == "d" { "0" } else { size };

        let mut entry = json!({
            "name": name,
            "type": file_type,
            "size": size,
            "time": format!("{} {}", date, time),
            "permissions": permissions,
            "owner": owner,
            "group": group,
            "links": links
        });
        if let Some(target) = link_target {
            entry["link_target"] = json!(target);
        }
        entries.push(entry);
    }

    for entry in &mut entries {
        let Some(name) = entry.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        if resolved_directories.contains(name) {
            entry["points_to_dir"] = json!(true);
        }
    }

    if entries.is_empty() && saw_invalid_row {
        return Err(output.trim().to_string());
    }

    Ok(entries)
}

fn is_valid_ls_permissions(permissions: &str) -> bool {
    let bytes = permissions.as_bytes();
    // type + 9 mode bits, optionally followed by ACL (+) and/or SELinux (.) markers
    if bytes.len() < 10 || bytes.len() > 12 {
        return false;
    }
    match bytes[0] {
        b'd' | b'l' | b'c' | b'b' | b'p' | b's' | b'-' => {},
        _ => return false,
    }
    for &byte in &bytes[1..10] {
        match byte {
            b'r' | b'w' | b'x' | b'-' | b's' | b'S' | b't' | b'T' => {},
            _ => return false,
        }
    }
    bytes[10..].iter().all(|&byte| byte == b'+' || byte == b'.')
}
