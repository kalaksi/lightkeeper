/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};

use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;
use crate::utils::ShellCommand;
use crate::host::HostSetting;

#[monitoring_module(
    name="storage-lvm-volume-group",
    version="0.0.1",
    description="Provides information about LVM volume groups.",
    uses_sudo=true,
)]
pub struct VolumeGroup {
}

impl Module for VolumeGroup {
    fn new(_settings: &HashMap<String, String>) -> Self {
        VolumeGroup {
        }
    }
}

impl MonitoringModule for VolumeGroup {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Volume Groups"),
            category: String::from("storage"),
            use_multivalue: true,
            use_without_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec![ "vgs", "--separator", "|", "--options", "vg_name,vg_attr,vg_size,vg_free", "--units", "h" ]);
            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.message.is_empty() && response.return_code == 0 {
            return Ok(DataPoint::empty());
        }
        else if response.is_command_not_found() {
            return Ok(DataPoint::value_with_level("LVM not available".to_string(), crate::enums::Criticality::NotAvailable));
        }

        let mut result = DataPoint::empty();

        let lines = response.message.lines().skip(1);
        for line in lines {
            let parts = line.split("|").collect::<Vec<&str>>();
            let [vg_name, vg_attr, vg_size, vg_free, ..] = parts.as_slice()
            else {
                return Ok(DataPoint::invalid_response());
            };
            let vg_name = vg_name.trim_start().to_string();
            let vg_attr = vg_attr.to_string();
            let vg_size = vg_size.to_string();
            let vg_free = vg_free.to_string();

            let mut data_point = DataPoint::labeled_value(vg_name.clone(), String::from("OK"));
            data_point.description = format!("free: {} / {}", vg_free, vg_size);

            if vg_attr.chars().nth(0) == Some('r') {
                data_point.tags.push(String::from("Read-only"));
            }

            if vg_attr.chars().nth(5) == Some('p') {
                data_point.criticality = crate::enums::Criticality::Error;
                data_point.value = String::from("Partial");
            }

            data_point.command_params = vec![vg_name];
            result.multivalue.push(data_point);
        }

        Ok(result)
    }
}