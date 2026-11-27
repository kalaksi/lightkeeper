/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */


use std::collections::{HashMap, HashSet};

use crate::enums::Criticality;
use crate::error::LkError;
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
    uses_sudo=true,
    access="read",
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
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Packages"),
            category: String::from("packages"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_messages(&self, host: Host, _result: DataPoint) -> Result<Vec<String>, LkError> {
        if !uses_dnf(&host) {
            return Err(LkError::not_implemented());
        }

        let color_never = dnf_uses_color_never(&host);
        Ok(vec![
            dnf_check_update_command(&host, false, color_never),
            dnf_check_update_command(&host, true, color_never),
        ])
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();

        if host.platform.is_same_or_greater(Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(Flavor::Ubuntu, "20") {
            command.arguments(vec!["apt", "list", "--upgradable"]);
            Ok(command.to_string())
        }
        // On NixOS and CoreOS, things are more complicated.
        // No easy way to get version update information, so ignoring for now.
        else if host.platform.os_flavor == Flavor::NixOS ||
                host.platform.is_variant(Flavor::Fedora, "coreos") {
            Ok(String::new())
        }
        else if host.platform.os_flavor == Flavor::Alpine {
            command.use_sudo = !host.is_root;
            command.arguments(vec!["apk", "list", "--upgradable"]);
            Ok(command.to_string())
        }
        else if uses_dnf(&host) {
            Err(LkError::not_implemented())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_responses(&self, host: Host, responses: Vec<ResponseMessage>, _result: DataPoint) -> Result<DataPoint, String> {
        if !uses_dnf(&host) {
            return Err(String::new());
        }

        let all_response = responses.get(0).ok_or_else(|| String::from("Missing dnf check-update response"))?;
        let security_response = responses.get(1);

        // Uses exit code 100 if there are updates available.
        if all_response.is_error() && all_response.return_code != 100 {
            return Err(all_response.message.clone());
        }

        let security_packages = security_response
            .filter(|response| !response.is_error() || response.return_code == 100)
            .map(|response| dnf_package_names(&response.message))
            .unwrap_or_default();

        let mut result = DataPoint::empty();
        for (package_name, new_version, repository) in dnf_update_rows(&all_response.message) {
            let is_security = security_packages.contains(&package_name);
            let mut data_point = DataPoint::labeled_value(package_name.clone(), new_version);
            data_point.description = repository;
            data_point.command_params = vec![package_name];
            if is_security {
                data_point.criticality = Criticality::Warning;
                data_point.tags.push(String::from("Security"));
            }
            result.multivalue.push(data_point);
        }

        result.update_criticality_from_children();
        Ok(result)
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
                let (package_name, archives) = full_package.split_once('/')
                    .map(|(name, archives)| (name.to_string(), archives))
                    .unwrap_or((full_package.clone(), ""));
                let new_version = parts.next().unwrap_or_default().to_string();

                let old_version = string_manipulation::get_string_between(&line, "[upgradable from: ", "]")
                    .unwrap_or(String::from("unknown version"));
                let mut data_point = DataPoint::labeled_value(package_name, new_version);
                data_point.description = old_version;
                data_point.command_params = vec![package];
                if archives.contains("-security") {
                    data_point.criticality = Criticality::Warning;
                    data_point.tags.push(String::from("Security"));
                }
                result.multivalue.push(data_point);
            }
        }
        else if host.platform.os_flavor == Flavor::Alpine {
            // Example: rsync-3.2.3-r4 x86_64 {rsync} (GPL-3.0-or-later) [upgradable from: rsync-3.2.3-r2]
            let lines = response.message.lines().filter(|line| line.contains("[upgradable"));
            for line in lines {
                let package_name = string_manipulation::get_string_between(&line, "{", "}")
                    .ok_or(LkError::unexpected())?;
                let name_prefix = format!("{}-", package_name);
                let full_new = line.split_whitespace().next().ok_or(LkError::unexpected())?.to_string();
                let new_version = full_new.strip_prefix(&name_prefix).unwrap_or(&full_new).to_string();
                let full_old = string_manipulation::get_string_between(&line, "[upgradable from: ", "]")
                    .unwrap_or(String::from("unknown version"));
                let old_version = full_old.strip_prefix(&name_prefix).unwrap_or(&full_old).to_string();

                let mut data_point = DataPoint::labeled_value(package_name.clone(), new_version);
                data_point.description = old_version;
                data_point.command_params = vec![package_name];
                result.multivalue.push(data_point);
            }
        }
        else {
            return Err(String::from("Unsupported platform for package response"));
        }

        result.update_criticality_from_children();
        Ok(result)
    }
}

fn uses_dnf(host: &Host) -> bool {
    host.platform.is_same_or_greater(Flavor::CentOS, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
        host.platform.is_same_or_greater(Flavor::AlmaLinux, "8") ||
        host.platform.is_same_or_greater(Flavor::Rocky, "8") ||
        host.platform.is_same_or_greater(Flavor::AmazonLinux, "2023") ||
        (host.platform.os_flavor == platform_info::Flavor::Fedora &&
            !host.platform.is_variant(platform_info::Flavor::Fedora, "coreos"))
}

fn dnf_uses_color_never(host: &Host) -> bool {
    host.platform.is_same_or_greater(Flavor::CentOS, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
        host.platform.is_same_or_greater(Flavor::AlmaLinux, "8") ||
        host.platform.is_same_or_greater(Flavor::Rocky, "8") ||
        host.platform.is_same_or_greater(Flavor::AmazonLinux, "2023")
}

fn dnf_check_update_command(host: &Host, security_only: bool, color_never: bool) -> String {
    let mut command = ShellCommand::new();
    command.use_sudo = !host.is_root;
    command.argument("dnf");
    command.argument("check-update");
    if security_only {
        command.argument("--security");
    }
    command.argument("--quiet");
    if color_never {
        command.argument("--color=never");
    }
    command.argument("--assumeno");
    command.to_string()
}

fn dnf_update_rows(message: &str) -> Vec<(String, String, String)> {
    let mut rows = Vec::new();
    for line in message.lines().filter(|line| !line.is_empty()) {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() < 3 {
            continue;
        }
        let package_name = tokens[0].to_string();
        if !package_name.contains('.') {
            continue;
        }
        rows.push((package_name, tokens[1].to_string(), tokens[2].to_string()));
    }
    rows
}

fn dnf_package_names(message: &str) -> HashSet<String> {
    dnf_update_rows(message).into_iter().map(|(name, _, _)| name).collect()
}
