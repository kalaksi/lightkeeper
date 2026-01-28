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
    name="podman-inspect",
    version="0.0.1",
    description="Inspects a Podman container.",
    uses_sudo=true,
)]
pub struct Inspect;

impl Module for Inspect {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Inspect { }
    }
}

impl CommandModule for Inspect {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("podman-containers"),
            parent_id: String::from("podman-containers"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("search"),
            display_text: String::from("Inspect"),
            action: UIAction::TextView,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let target_id = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if !string_validation::is_alphanumeric_with(target_id, &"-_") {
            Err(LkError::invalid_parameter("Invalid container ID", target_id))
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            let socket_path = String::from("/run/podman/podman.sock");
            let url = format!("http://localhost/containers/{}/json?all=true", target_id);
            command.arguments(vec!["curl", "-s", "--unix-socket", &socket_path, &url]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new_info(response.message.clone()))
    }
}
