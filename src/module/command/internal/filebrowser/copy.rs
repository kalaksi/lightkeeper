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
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::enums;
use lightkeeper_module::command_module;

#[command_module(
    name="_internal-filebrowser-copy",
    version="0.0.1",
    description="Copy files or directories on the remote host with rsync.",
)]
pub struct FileBrowserCopy {
}

impl Module for FileBrowserCopy {
    fn new(_settings: &HashMap<String, String>) -> Self {
        FileBrowserCopy {
        }
    }
}

impl CommandModule for FileBrowserCopy {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("edit-copy"),
            display_text: String::from("Copy files"),
            depends_on_criticality: vec![enums::Criticality::Normal],
            action: UIAction::FollowOutput,
            tab_title: String::from("Copy"),
            parent_id: String::from("_internal-filebrowser-ls"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = false;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if parameters.len() < 2 {
                return Err(LkError::other("Destination and at least one source path required"));
            }
            let destination = parameters.first().ok_or(LkError::other("No destination specified"))?;
            let sources = &parameters[1..];
            if destination.is_empty() {
                return Err(LkError::other("Destination is empty"));
            }
            for source in sources {
                if source.is_empty() {
                    return Err(LkError::other("Empty source path"));
                }
            }
            let dest = if destination.ends_with('/') {
                destination.clone()
            }
            else {
                format!("{}/", destination)
            };
            let mut args: Vec<&str> = vec!["rsync", "-av", "--info=progress2", "--ignore-existing"];
            args.extend(sources.iter().map(String::as_str));
            args.push(dest.as_str());
            command.arguments(args);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            let progress = download::parse_rsync_progress(&response.message);
            Ok(CommandResult::new_partial(response.message_increment.clone(), progress))
        }
        else if response.return_code == 0 {
            Ok(CommandResult::new_hidden(response.message_increment.clone()))
        }
        else {
            Ok(CommandResult::new_hidden(response.message_increment.clone())
                .with_criticality(crate::enums::Criticality::Error))
        }
    }
}
