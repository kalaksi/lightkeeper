/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

use serde_derive::Deserialize;
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
    name="podman-image-remove",
    version="0.0.1",
    description="Removes a Podman image.",
    uses_sudo=true,
)]
pub struct Remove;

impl Module for Remove {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Remove { }
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
        command.use_sudo = true;

        if !string_validation::is_alphanumeric_with(target_id, ":-.") {
            Err(LkError::invalid_parameter("Invalid image ID", target_id))
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            let socket_path = String::from("/run/podman/podman.sock");
            let url = format!("http://localhost/images/{}", target_id);
            command.arguments(vec!["curl", "-s", "--unix-socket", &socket_path, "-X", "DELETE", &url]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.message.len() > 0 {
            if let Ok(deletion_details) = serde_json::from_str::<Vec<DeletionDetails>>(&response.message) {
                let untagged_count = deletion_details.iter().filter(|details| details.untagged.is_some()).count();
                let deleted_count = deletion_details.iter().filter(|details| details.deleted.is_some()).count();
                let response_message = format!("{} layers untagged, {} layers deleted", untagged_count, deleted_count);
                return Ok(CommandResult::new_info(response_message));
            }
            else if let Ok(podman_response) = serde_json::from_str::<ErrorMessage>(&response.message) {
                return Ok(CommandResult::new_error(podman_response.message));
            }
        }
        Ok(CommandResult::new_info(response.message.clone()))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DeletionDetails {
    untagged: Option<String>,
    deleted: Option<String>,
}

#[derive(Deserialize)]
struct ErrorMessage {
    message: String,
}
