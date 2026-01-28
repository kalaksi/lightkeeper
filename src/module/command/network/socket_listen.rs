/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use crate::error::LkError;
use crate::frontend;
use crate::host::*;
use crate::module::command::UIAction;
use crate::module::connection::ResponseMessage;
use crate::module::*;
use crate::module::command::*;
use crate::utils::ShellCommand;
use lightkeeper_module::command_module;

#[command_module(
    name="network-socket-listen",
    version="0.0.1",
    description="Show listening TCP and UDP sockets.",
    uses_sudo=true,
)]
pub struct SocketListen;

impl Module for SocketListen {
    fn new(_settings: &HashMap<String, String>) -> Self {
        SocketListen { }
    }
}

impl CommandModule for SocketListen {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("network"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("view-document"),
            display_text: String::from("Show listening sockets"),
            tab_title: String::from("Listening sockets"),
            action: UIAction::TextDialog,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parameters: Vec<String>) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec!["netstat", "-tulnp"]);
            Ok(command.to_string())
        }
        else {
            return Err(LkError::unsupported_platform());
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.is_error() {
            return Err(response.message.clone());
        }
        let monospaced_markdown = format!("```\n{}\n```", response.message);
        Ok(CommandResult::new_hidden(monospaced_markdown))
    }
}