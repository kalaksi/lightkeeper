/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use serde::Deserialize;
use serde_json;

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
    name="docker-rm",
    version="0.0.1",
    description="Removes a Docker container.",
    uses_sudo=true,
)]
pub struct Rm;

impl Module for Rm {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Rm { }
    }
}

impl CommandModule for Rm {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-containers"),
            parent_id: String::from("docker-containers"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("delete"),
            display_text: String::from("Remove"),
            confirmation_text: String::from("Really remove container?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let target_id = parameters.first().unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = true;

        if !string_validation::is_alphanumeric_with(target_id, &"-_") {
            Err(LkError::invalid_parameter("Invalid container ID", target_id))
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            let url = format!("http://localhost/containers/{}", target_id);
            command.arguments(vec!["curl", "-s", "--unix-socket", "/var/run/docker.sock", "-X", "DELETE", &url]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            if let Ok(docker_response) = serde_json::from_str::<ErrorMessage>(&response.message) {
                return Ok(CommandResult::new_error(docker_response.message));
            }
        }
        Ok(CommandResult::new_info(response.message.clone()))
    }
}

#[derive(Deserialize)]
struct ErrorMessage {
    message: String,
}
