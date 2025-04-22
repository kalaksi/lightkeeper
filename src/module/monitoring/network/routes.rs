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

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") {
            Ok(String::from("/sbin/ip route ls"))
        }
        else if host.platform.os_flavor == platform_info::Flavor::Alpine {
            Ok(String::from("ip route"))
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("ip route ls"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();

        let lines = response.message.lines();
        for line in lines {
            // Get substring before word "proto".
            let route = line.split("proto").next().unwrap_or_default().trim().to_string();
            let mut parts = route.split("dev");
            let subnet = parts.next().unwrap_or_default().trim().to_string();
            let interface = parts.next().unwrap_or_default().trim().to_string();

            if subnet.is_empty() && interface.is_empty() {
                return Err(format!("Couldn't parse route: {}", line));
            }

            let data_point = DataPoint::labeled_value(subnet, interface);
            result.multivalue.push(data_point);
        }
        Ok(result)
    }
}