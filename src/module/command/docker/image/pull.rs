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
    name="docker-image-pull",
    version="0.0.1",
    description="Pulls a Docker image from a registry.",
    uses_sudo=true,
)]
pub struct Pull;

impl Module for Pull {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Pull { }
    }
}

impl CommandModule for Pull {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-images"),
            parent_id: String::from("docker-images"),
            secondary_parent_id: String::from("docker-image-updates"),
            depends_on_no_tags: vec![String::from("Local")],
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("download"),
            display_text: String::from("Pull"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let image_repo_tag = parameters.get(1).unwrap_or(&String::new()).clone();

        if image_repo_tag.is_empty() {
            return Err(LkError::other("Image has no tag and can not be pulled."));
        }

        if !string_validation::is_alphanumeric_with(&image_repo_tag, ":/._-") {
            return Err(LkError::invalid_parameter("Invalid image name", &image_repo_tag));
        }

        let mut command = ShellCommand::new();
        command.use_sudo = !host.is_root;

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["docker", "pull", &image_repo_tag]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_hidden(&response.message))
        } else {
            Err(response.message.clone())
        }
    }
}
