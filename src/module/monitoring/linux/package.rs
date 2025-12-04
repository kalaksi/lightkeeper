/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;

use crate::enums::Criticality;
use crate::error::LkError;
use crate::host::HostSetting;
use crate::module::connection::ResponseMessage;
use crate::module::platform_info::Flavor;
use crate::{ Host, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;
use crate::utils::{ShellCommand, string_manipulation};

#[monitoring_module(
    name="package",
    version="0.0.1",
    description="Lists system packages that have an update available.",
)]
pub struct Package;

impl Module for Package {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Package { }
    }
}

impl MonitoringModule for Package {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Packages"),
            category: String::from("packages"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.is_same_or_greater(Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(Flavor::Ubuntu, "20") {

            command.arguments(vec!["apt", "list", "--upgradable"]);
            Ok(command.to_string())
        }
        else if host.platform.is_same_or_greater(Flavor::CentOS, "8") ||
                host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") {
            command.arguments(vec!["dnf", "check-update", "--quiet", "--color=never", "--assumeno"]);
            Ok(command.to_string())
        }
        else if host.platform.os_flavor == platform_info::Flavor::Fedora {
            command.arguments(vec!["dnf", "check-update", "--quiet", "--assumeno"]);
            Ok(command.to_string())
        }
        else if host.platform.is_same_or_greater(Flavor::NixOS, "20") {
            // On NixOS, things are more complicated. No easy way to get version update information, so ignoring for now.
            Ok(String::new())

        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.is_error() {
            return Err(response.message);
        }

        let mut result = DataPoint::empty();

        if host.platform.is_same_or_greater(Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(Flavor::Ubuntu, "20") {
            let lines = response.message.lines().filter(|line| line.contains("[upgradable"));
            for line in lines {
                let mut parts = line.split_whitespace();
                let full_package = parts.next().ok_or(LkError::unexpected())?.to_string();
                let package = full_package.split(',').nth(0).map(|s| s.to_string())
                                          .unwrap_or(full_package.clone());
                let package_name = full_package.split('/').next().ok_or(LkError::unexpected())?.to_string();
                let new_version = parts.next().unwrap_or_default().to_string();
                // let arch = parts.next()?.to_string();

                let old_version = string_manipulation::get_string_between(&line, "[upgradable from: ", "]")
                    .unwrap_or(String::from("unknown version"));
                let mut data_point = DataPoint::labeled_value(package_name, new_version);
                data_point.description = old_version;
                data_point.command_params = vec![package];
                result.multivalue.push(data_point);
            }
        }
        else if host.platform.is_same_or_greater(Flavor::CentOS, "8") ||
                host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
                host.platform.os_flavor == platform_info::Flavor::Fedora {

            let lines = response.message.lines().filter(|line| !line.is_empty());
            for line in lines {
                let mut parts = line.split_whitespace();

                let package_name = parts.next().unwrap_or_default().to_string();
                let new_version = parts.next().unwrap_or_default().to_string();
                let repository = parts.next().unwrap_or_default().to_string();

                let mut data_point = DataPoint::labeled_value(package_name.clone(), new_version);
                data_point.description = repository;
                data_point.command_params = vec![package_name];
                result.multivalue.push(data_point);
            }
        }

        Ok(result)
    }
}