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
use crate::module::command::systemd::user::supports_systemctl_user;
use crate::utils::string_validation;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="systemd-user-service-start",
    version="0.0.1",
    description="Starts a SystemD user service.",
)]
pub struct Start;

impl Module for Start {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Start { }
    }
}

impl CommandModule for Start {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-user-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("start"),
            display_text: String::from("Start"),
            // Only displayed if the service isn't running.
            depends_on_no_tags: vec![String::from("active")],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let service = parameters.first().unwrap();
        let mut command = ShellCommand::new();

        if !string_validation::is_alphanumeric_with(service, "-_.@\\") ||
            string_validation::begins_with_dash(service)
        {
            Err(LkError::other_p("Invalid unit name", service))
        }
        else if supports_systemctl_user(&host) {
            command.arguments(vec!["systemctl", "--user", "start", service]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if !response.message.is_empty() {
            Ok(CommandResult::new_error(response.message.clone()))
        }
        else {
            Ok(CommandResult::new_info(response.message.clone()))
        }
    }
}
