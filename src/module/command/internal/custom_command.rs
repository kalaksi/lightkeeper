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
    name="_custom-command",
    version="0.0.1",
    description="Internal module for user-defined custom commands.",
)]
pub struct CustomCommand;

impl Module for CustomCommand {
    fn new(_settings: &HashMap<String, String>) -> Self {
        CustomCommand {
        }
    }
}

impl CommandModule for CustomCommand {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("_custom-command"),
            display_icon: String::from("refresh"),
            display_text: String::from("Custom command"),
            action: UIAction::FollowOutput,
            tab_title: String::from("Custom command"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = parameters.get(0).ok_or(LkError::other("No command specified"))?.clone();

        if command.ends_with("\n") {
            command.pop();
        }
        if command.ends_with("\r") {
            command.pop();
        }

        if host.platform.os == platform_info::OperatingSystem::Linux {
            let shell_command = ShellCommand::new_from(vec!["sh", "-c", &command]);
            Ok(shell_command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            Ok(CommandResult::new_partial(response.message_increment.clone(), 1))
        }
        else {
            if response.return_code == 0 {
                Ok(CommandResult::new_hidden(response.message_increment.clone()))
            }
            else {
                Ok(CommandResult::new_hidden(response.message_increment.clone())
                                 .with_criticality(crate::enums::Criticality::Error))
            }
        }
    }
}