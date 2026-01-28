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
    name="storage-lvm-lvremove",
    version="0.0.1",
    description="Removes an LVM logical volume.",
    uses_sudo=true,
)]
pub struct LVRemove {
}

impl Module for LVRemove {
    fn new(_settings: &HashMap<String, String>) -> Self {
        LVRemove {
        }
    }
}

impl CommandModule for LVRemove {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("storage"),
            parent_id: String::from("storage-lvm-logical-volume"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("delete"),
            display_text: String::from("Remove"),
            confirmation_text: String::from("Are you sure you want to remove this logical volume?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let lv_path = parameters.get(0).unwrap();
        let _vg_name = parameters.get(1).unwrap();
        let _lv_name = parameters.get(2).unwrap();
        let _lv_size = parameters.get(3).unwrap();

        let mut command = ShellCommand::new();
        command.use_sudo = true;

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            command.arguments(vec!["lvremove", "-y", lv_path]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 && response.message.contains("successfully removed"){
            Ok(CommandResult::new_info(String::new()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}