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
use serde_json::{json, Value};

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
        let mut command = ShellCommand::new();
        command.use_sudo = host.should_use_sudo();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            let path = parameters.first().ok_or(LkError::other("No path specified"))?.as_str();
            // --full-time works on both GNU coreutils and BusyBox; --time-style does not.
            command.arguments(vec![
                "ls",
                "-lAL",
                "--group-directories-first",
                "--color=never",
                "--full-time",
                path
            ]);
        }
        else {
            return Err(LkError::unsupported_platform());
        }

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code != 0 {
            let msg = if response.message.trim().is_empty() {
                format!("Failed to list directory (exit code {})", response.return_code)
            }
            else {
                response.message.clone()
            };
            return Ok(CommandResult::new_error(msg));
        }

        let entries = parse_ls_output(&response.message)?;
        let json_result = json!({
            "entries": entries
        });
        Ok(CommandResult::new_hidden(json_result.to_string()))
    }
}

pub fn parse_ls_output(output: &str) -> Result<Vec<Value>, String> {
    let mut entries = Vec::new();
    let mut lines = output.lines().map(str::trim).filter(|line| !line.is_empty());

    let Some(first_line) = lines.next() else {
        return Ok(entries);
    };

    // Valid `ls -l` directory listings always start with "total N". Error messages do not;
    // without this check they get mis-parsed as file rows and spread across UI columns.
    if !first_line.starts_with("total ") {
        return Err(output.trim().to_string());
    }

    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // permissions links owner group size date time [timezone] name...
        if parts.len() < 8 {
            log::warn!("Invalid line: {}", line);
            continue;
        }

        let permissions = parts[0];
        let links = parts[1];
        let owner = parts[2];
        let group = parts[3];
        let size = parts[4];
        let date = parts[5];
        let time = parts[6];

        if !is_valid_ls_permissions(permissions) {
            return Err(output.trim().to_string());
        }

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

        // Everything from name_start onwards is the name.
        // This also converts any other whitespace characters to spaces.
        let name = parts[name_start..].join(" ");

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

        // Merge date and time into a single time field
        let time_combined = format!("{} {}", date, time);

        entries.push(json!({
            "name": name,
            "type": file_type,
            "size": size,
            "time": time_combined,
            "permissions": permissions,
            "owner": owner,
            "group": group,
            "links": links
        }));
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