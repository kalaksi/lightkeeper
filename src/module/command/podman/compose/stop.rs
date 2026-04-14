/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="podman-compose-stop",
    version="0.0.1",
    description="Stops podman-compose projects or services.",
    uses_sudo=true,
    settings={
        as_root => "Run podman with sudo as root. Default: true. If false, run as the SSH user (rootless)."
    }
)]
pub struct Stop {
    as_root: bool,
}

impl Module for Stop {
    fn new(settings: &HashMap<String, String>) -> Stop {
        Stop {
            as_root: settings.get("as_root").and_then(|value| Some(value == "true")).unwrap_or(true),
        }
    }
}

impl CommandModule for Stop {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("podman-compose"),
            parent_id: String::from("podman-compose"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("stop"),
            display_text: String::from("Stop"),
            confirmation_text: String::from("Really stop container?"),
            // Hide if container is stopped (status starts with "Exited").
            depends_on_no_value: vec![String::from("Exited")],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let compose_file = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = self.as_root;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["podman", "compose", "-f", compose_file, "stop"]);
            if let Some(service_name) = parameters.get(2) {
                command.argument(service_name);
            }
        }
        else {
            return Err(LkError::unsupported_platform())
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &connection::ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::default())
        } else {
            Err(response.message.clone())
        }
    }
}
