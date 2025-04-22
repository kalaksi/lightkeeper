/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use chrono;

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
    name="storage-lvm-snapshot",
    version="0.0.1",
    description="Creates a snapshot of an LVM logical volume.",
    settings={
      snapshot_suffix => "The suffix to append to the snapshot name. Defaults to '_snapshot_$TIME'."
    }
)]
pub struct Snapshot {
    pub snapshot_suffix: String,
}

impl Module for Snapshot {
    fn new(settings: &HashMap<String, String>) -> Self {
        Snapshot {
            snapshot_suffix: settings.get("snapshot_suffix").unwrap_or(&String::from("_snapshot_$TIME")).clone(),
        }
    }
}

impl CommandModule for Snapshot {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("storage"),
            parent_id: String::from("storage-lvm-logical-volume"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("copy"),
            display_text: String::from("Create a snapshot"),
            depends_on_no_tags: vec![String::from("Snapshot")],
            user_parameters: vec![
                frontend::UserInputField::decimal_number_with_units("Snapshot size", "3G", &[
                    "r", "R", "b", "B", "s", "S", "k", "K", "m", "M", "g", "G", "t", "T", "p", "P", "e", "E"
                ]),
            ],
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, parameters: Vec<String>) -> Result<String, LkError> {
        let lv_path = parameters.get(0).unwrap();
        let _vg_name = parameters.get(1).unwrap();
        let lv_name = parameters.get(2).unwrap();
        let _lv_size = parameters.get(3).unwrap();
        let new_size = crate::utils::remove_whitespace(parameters.get(4).unwrap());

        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let snapshot_suffix_with_timestamp = self.snapshot_suffix.replace("$TIME", &timestamp);
        let snapshot_name = format!("{}{}", lv_name, snapshot_suffix_with_timestamp);

        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&crate::host::HostSetting::UseSudo);

        if !string_validation::is_numeric_with_unit(&new_size, &self.get_display_options().user_parameters[0].units) {
            Err(LkError::other_p("Invalid size", &new_size))
        }
        else if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
                host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
                host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
                host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            command.arguments(vec!["lvcreate", "--snapshot", "--name", &snapshot_name, "--size", &new_size, lv_path]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }

    }

    fn process_response(&self, _host: Host, response: &ResponseMessage) -> Result<CommandResult, String> {
        if response.return_code == 0 {
            Ok(CommandResult::new_info(String::new()))
        }
        else {
            Ok(CommandResult::new_error(response.message.clone()))
        }
    }
}