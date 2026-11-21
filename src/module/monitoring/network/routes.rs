/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
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

#[monitoring_module(
    name="network-routes",
    version="0.0.1",
    description="Provides routing table information.",
    settings={
    }
)]
pub struct Routes {
}

impl Module for Routes {
    fn new(_: &HashMap<String, String>) -> Self {
        Routes {
        }
    }
}

impl MonitoringModule for Routes {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Routes"),
            category: String::from("network"),
            use_multivalue: true,
            use_without_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_messages(&self, host: Host, _result: DataPoint) -> Result<Vec<String>, LkError> {
        // BusyBox ip does not accept "route ls"; plain "route" works on both BusyBox and iproute2.
        // RHEL-family often lacks /sbin on non-root PATH, so use the absolute path there.
        // Elsewhere prefer bare "ip" so NixOS (no /sbin/ip) and similar still work.
        let ip = if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::AlmaLinux, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Rocky, "7") {
            "/sbin/ip"
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            "ip"
        }
        else {
            return Err(LkError::unsupported_platform());
        };

        Ok(vec![format!("{} route", ip), format!("{} -6 route", ip)])
    }

    fn process_responses(&self, _host: Host, responses: Vec<ResponseMessage>, _result: DataPoint) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();
        let mut saw_success = false;
        let mut last_error = None;

        for response in responses {
            if response.is_command_not_found() {
                continue;
            }
            if response.is_error() {
                last_error = Some(response.message);
                continue;
            }

            saw_success = true;
            for line in response.message.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                // Get substring before word "proto".
                let route = line.split("proto").next().unwrap_or_default().trim().to_string();
                let mut parts = route.split("dev");
                let subnet = parts.next().unwrap_or_default().trim().to_string();
                let interface = parts.next().unwrap_or_default().trim().to_string();

                if subnet.is_empty() && interface.is_empty() {
                    return Err(format!("Couldn't parse route: {}", line));
                }

                result.multivalue.push(DataPoint::labeled_value(subnet, interface));
            }
        }

        if !saw_success {
            if let Some(message) = last_error {
                return Err(message);
            }
            return Ok(DataPoint::not_available("ip not available"));
        }

        Ok(result)
    }
}
