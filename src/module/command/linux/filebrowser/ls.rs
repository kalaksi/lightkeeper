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
use serde_json::{json, Value};

#[command_module(
    name="linux-filebrowser-ls",
    version="0.0.1",
    description="List files and directories.",
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
            depends_on_criticality: vec![enums::Criticality::Normal],
            action: UIAction::FileBrowser,
            tab_title: String::from("File browser"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = false;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            let path = parameters.first().ok_or(LkError::other("No path specified"))?.as_str();
            command.arguments(vec!["ls", "-lAh", "--group-directories-first", "--color=never", "--time-style=long-iso", path]);
        }
        else {
            return Err(LkError::unsupported_platform());
        }

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        let entries = parse_ls_output(&response.message)?;
        
        let json_result = json!({
            "entries": entries
        });
        
        Ok(CommandResult::new_hidden(json_result.to_string()))
    }
}

fn parse_ls_output(output: &str) -> Result<Vec<Value>, String> {
    let mut entries = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    
    // Skip the first line which contains "total X"
    for line in lines.iter().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        let [permissions, links, owner, group, size, date, time, name_parts @ ..] = parts.as_slice()
        else {
            log::warn!("Invalid line: {}", line);
            continue;
        };
        
        // Handle names with spaces - everything from index 7 onwards is the name
        let name = name_parts.join(" ");
        
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
        
        entries.push(json!({
            "name": name,
            "type": file_type,
            "size": size,
            "date": date,
            "time": time,
            "permissions": permissions,
            "owner": owner,
            "group": group,
            "links": links
        }));
    }
    
    Ok(entries)
}

