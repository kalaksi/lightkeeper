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
use crate::utils::string_validation;
use lightkeeper_module::command_module;

#[command_module(
    name="podman-image-remove",
    version="0.0.1",
    description="Removes a Podman image.",
    uses_sudo=true,
    settings={
        as_root => "Run podman with sudo as root. Default: true. If false, run as the SSH user (rootless)."
    }
)]
pub struct Remove {
    as_root: bool,
}

impl Module for Remove {
    fn new(settings: &HashMap<String, String>) -> Self {
        Remove {
            as_root: settings.get("as_root").and_then(|value| Some(value == "true")).unwrap_or(true),
        }
    }
}

impl CommandModule for Remove {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("podman-images"),
            parent_id: String::from("podman-images"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("delete"),
            display_text: String::from("Delete"),
            confirmation_text: String::from("Really remove image?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let target_id = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = self.as_root;

        if !string_validation::is_alphanumeric_with(target_id, ":-.") {
            Err(LkError::invalid_parameter("Invalid image ID", target_id))
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["podman", "rmi", "-f", target_id]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code != 0 {
            return Ok(CommandResult::new_error(response.message.trim()));
        }
        let text = response.message.trim();
        if text.is_empty() {
            Ok(CommandResult::new_info("Removed."))
        }
        else {
            Ok(CommandResult::new_info(text))
        }
    }
}
