/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
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
use serde_json::json;

#[command_module(
    name="_internal-filebrowser-du",
    version="0.0.1",
    description="Calculate disk usage for files and directories.",
    uses_sudo=true,
)]
pub struct FileBrowserDu {
}

impl Module for FileBrowserDu {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserDu {}
    }
}

impl CommandModule for FileBrowserDu {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("search"),
            display_text: String::from("Calculate size"),
            parent_id: String::from("_internal-filebrowser-ls"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }
        if parameters.is_empty() {
            return Err(LkError::other("No paths specified"));
        }

        let paths: Vec<String> = parameters.iter()
            .map(|path| {
                let trimmed = path.trim_end_matches('/');
                if trimmed.is_empty() { path.clone() } else { trimmed.to_string() }
            })
            .collect();
        for path in &paths {
            if path.is_empty() {
                return Err(LkError::other("Empty path"));
            }
        }

        let mut command = ShellCommand::new();
        command.use_sudo = host.should_use_sudo();
        // -k is BusyBox-compatible; sizes are kept in kB to match the file browser column.
        let mut args = vec!["du", "-kxs", "--"];
        args.extend(paths.iter().map(String::as_str));
        command.arguments(args);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        let sizes = parse_du_output(&response.message);
        if sizes.is_empty() && response.return_code != 0 {
            let msg = if response.message.trim().is_empty() {
                format!("Failed to calculate size (exit code {})", response.return_code)
            }
            else {
                response.message.clone()
            };
            return Ok(CommandResult::new_error(msg));
        }

        Ok(CommandResult::new_hidden(json!({ "sizes": sizes }).to_string()))
    }
}

pub fn parse_du_output(output: &str) -> Vec<serde_json::Value> {
    let mut sizes = Vec::new();
    for line in output.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Some((size, path)) = line.split_once(|c: char| c.is_whitespace()) else {
            continue;
        };
        let path = path.trim_start();
        if path.is_empty() || size.parse::<u64>().is_err() {
            continue;
        }
        sizes.push(json!({
            "path": path,
            "size": size,
        }));
    }
    sizes
}
