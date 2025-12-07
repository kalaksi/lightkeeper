/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;
use crate::enums::Criticality;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::utils::ShellCommand;
use crate::{
    Host,
    frontend,
};

use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="network-dns",
    version="0.0.1",
    description="Provides information about DNS settings.",
    settings={
    }
)]
pub struct Dns {
}

impl Module for Dns {
    fn new(_: &HashMap<String, String>) -> Self {
        Dns {
        }
    }
}

impl MonitoringModule for Dns {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("DNS settings"),
            category: String::from("network"),
            use_multivalue: true,
            use_without_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_messages(&self, host: Host, _parent_result: DataPoint) -> Result<Vec<String>, LkError> {
        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "10") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {

            let command_resolvconf = ShellCommand::new_from(vec!["grep", "-E", "^nameserver", "/etc/resolv.conf"]);
            // TODO: use busctl instead?
            let command_resolvectl = ShellCommand::new_from(vec!["resolvectl", "dns"]);
            Ok(vec![command_resolvconf.to_string(), command_resolvectl.to_string()])
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_responses(&self, _host: Host, responses: Vec<ResponseMessage>, _parent_result: DataPoint) -> Result<DataPoint, String> {
        // Errors can be somewhat expected depending on what resources are available or how host is configured..
        let mut result = DataPoint::empty();

        if let Some(resolvconf_response) = responses.get(0) {
            if resolvconf_response.is_success() {
                // Lines will always start with "nameserver" so never empty or commented out.
                for line in resolvconf_response.message.lines() {
                    let parts = line.split(' ')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<&str>>();

                    let [_prefix, server_address, ..]  = parts.as_slice()
                    else {
                        continue;
                    };

                    let mut datapoint = DataPoint::label(server_address);
                    datapoint.description = String::from("resolv.conf");
                    result.multivalue.push(datapoint);
                }
            }
        }

        // resolvectl-command might have failed and response missing.
        if let Some(resolvectl_response) = &responses.get(1) {
            if resolvectl_response.is_success() {
                for line in resolvectl_response.message.lines() {
                    if line.starts_with("Link") {
                        let mut parts = line.split("): ");
                        let dns_servers = parts.nth(1).unwrap_or_default()
                                            .split_whitespace();

                        for dns_server in dns_servers {
                            let mut datapoint = DataPoint::label(dns_server);
                            datapoint.description = String::from("systemd-resolved");
                            result.multivalue.push(datapoint);
                        }
                    }
                }
            }
        }

        if result.multivalue.is_empty() {
            result.multivalue.push(DataPoint::labeled_value_with_level(
                ".conf".to_string(),
                "No DNS servers found".to_string(),
                Criticality::Critical
            ));
        }

        result.update_criticality_from_children();

        Ok(result)
    }
}