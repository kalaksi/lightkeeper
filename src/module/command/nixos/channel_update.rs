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
    name="nixos-channel-update",
    version="0.0.1",
    description="Updates the nix channel.",
    uses_sudo=true,
)]
pub struct ChannelUpdate;

impl Module for ChannelUpdate {
    fn new(_settings: &HashMap<String, String>) -> ChannelUpdate {
        ChannelUpdate {
        }
    }
}

impl CommandModule for ChannelUpdate {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("nixos"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("update"),
            display_text: String::from("Update nix channel"),
            tab_title: String::from("nix-channel --update"),
            action: UIAction::FollowOutput,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = true;

        if host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {
            command.arguments(vec!["nix-channel", "--update"]);
        }
        else {
            return Err(LkError::unsupported_platform());
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_partial {
            let progress = 10;
            Ok(CommandResult::new_partial(response.message_increment.clone(), progress))
        }
        else {
            if response.return_code == 0 {
                Ok(CommandResult::new_hidden(response.message_increment.clone()))
            }
            else {
                Ok(CommandResult::new_hidden(response.message_increment.clone())
                                 .with_criticality(crate::enums::Criticality::Error))
            }
        }
    }
}