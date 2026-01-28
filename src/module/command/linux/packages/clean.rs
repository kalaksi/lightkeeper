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
    name="linux-packages-clean",
    version="0.0.1",
    description="Cleans the system's package cache.",
    uses_sudo=true,
)]
pub struct Clean;

impl Module for Clean {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Self { }
    }
}

impl CommandModule for Clean {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("packages"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("clear"),
            display_text: String::from("Clean package cache"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = true;

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") {
            command.arguments(vec!["apt-get", "clean"]);
        }
        else if host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
                host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") {
            command.arguments(vec!["dnf", "clean", "all"]);
        }
        else {
            return Err(LkError::unsupported_platform());
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_success() {
            Ok(CommandResult::new_info("Package cache cleaned"))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}