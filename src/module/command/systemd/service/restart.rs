/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use crate::enums::Criticality;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use crate::utils::string_validation;
use lightkeeper_module::command_module;

#[command_module(
    name="systemd-service-restart",
    version="0.0.1",
    description="Restarts a SystemD service.",
)]
pub struct Restart;

impl Module for Restart {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Restart { }
    }
}

impl CommandModule for Restart {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("systemd"),
            parent_id: String::from("systemd-service"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("refresh"),
            display_text: String::from("Restart"),
            // Only displayed if the service is running.
            depends_on_criticality: vec![Criticality::Normal, Criticality::Info, Criticality::Warning],
            confirmation_text: String::from("Really restart service?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let service = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if !string_validation::is_alphanumeric_with(service, "-_.@\\") ||
            string_validation::begins_with_dash(service){

            Err(LkError::other_p("Invalid unit name", service))
        }
        else if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
            host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
            host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
            host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
            host.platform.is_same_or_greater(platform_info::Flavor::Fedora, "22") ||
            host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            command.arguments(vec!["systemctl", "restart", service]);
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