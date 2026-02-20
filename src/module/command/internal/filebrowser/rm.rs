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

#[command_module(
    name="_internal-filebrowser-rm",
    version="0.0.1",
    description="Remove files on the remote host.",
)]
pub struct FileBrowserRm {
}

impl Module for FileBrowserRm {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserRm {}
    }
}

impl CommandModule for FileBrowserRm {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Hidden,
            display_icon: String::from("edit-delete"),
            display_text: String::from("Remove files"),
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
        for path in &parameters {
            if path.is_empty() {
                return Err(LkError::other("Empty path in list"));
            }
        }

        let mut command = ShellCommand::new();
        command.use_sudo = false;
        let mut args = vec!["rm", "--"];
        args.extend(parameters.iter().map(String::as_str));
        command.arguments(args);

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_hidden(response.message_increment.clone()))
        }
        else {
            Ok(CommandResult::new_hidden(response.message_increment.clone())
                .with_criticality(crate::enums::Criticality::Error))
        }
    }
}
