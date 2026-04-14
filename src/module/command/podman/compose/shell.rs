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
use crate::enums;
use lightkeeper_module::command_module;

#[command_module(
    name="podman-compose-shell",
    version="0.0.1",
    description="Opens a shell inside a Podman compose managed container.",
    uses_sudo=true,
    settings={
        as_root => "Run podman with sudo as root. Default: true. If false, run as the SSH user (rootless)."
    }
)]
pub struct Shell {
    as_root: bool,
}

impl Module for Shell {
    fn new(settings: &HashMap<String, String>) -> Self {
        Shell {
            as_root: settings.get("as_root").and_then(|value| Some(value == "true")).unwrap_or(true),
        }
    }
}

impl CommandModule for Shell {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("podman-compose"),
            parent_id: String::from("podman-compose"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("terminal"),
            display_text: String::from("Open shell inside"),
            depends_on_criticality: vec![enums::Criticality::Normal],
            action: UIAction::Terminal,
            tab_title: String::from("Podman shell"),
            multivalue_level: 2,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let compose_file = parameters.first().unwrap();
        let service = parameters.get(2).unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = self.as_root;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["podman", "compose", "-f", compose_file, "exec", service,
                                   "/bin/sh", "-c", "test -e /bin/bash && /bin/bash || /bin/sh"]);
        }
        else {
            return Err(LkError::unsupported_platform())
        }

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new_info(response.message.clone()))
    }
}
