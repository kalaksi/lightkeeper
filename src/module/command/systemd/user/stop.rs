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
use crate::module::command::systemd::user::supports_systemctl_user;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
use lightkeeper_module::command_module;

#[command_module(
    name="systemd-user-service-stop",
    version="0.0.1",
    description="Stops a SystemD user service.",
)]
pub struct Stop;

impl Module for Stop {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Stop { }
    }
}

impl CommandModule for Stop {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-user-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("stop"),
            display_text: String::from("Stop"),
            // Only displayed if the service is running.
            depends_on_tags: vec![String::from("active")],
            confirmation_text: String::from("Really stop service?"),
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
            command.arguments(vec!["systemctl", "--user", "stop", service]);
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
