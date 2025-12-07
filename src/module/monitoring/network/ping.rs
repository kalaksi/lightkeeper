/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::utils::ShellCommand;
use crate::{ Host, enums::Criticality, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="ping",
    version="0.0.1",
    description="Measures average latency to host with ICMP echo request. Does not work with flatpak.",
    settings={
        count => "Amount of echo requests to send. Default: 2.",
        timeout => "Timeout in seconds. Default: 10."
    }
)]
pub struct Ping {
    count: u8,
    timeout: u8,
}

impl Module for Ping {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Ping {
            count: _settings.get("count").and_then(|value| value.parse().ok()).unwrap_or(2),
            timeout: _settings.get("timeout").and_then(|value| value.parse().ok()).unwrap_or(10),
        }
    }
}

impl MonitoringModule for Ping {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("local-command", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Ping"),
            category: String::from("network"),
            unit: String::from("ms"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            command.arguments(vec![
                "ping", "-c", self.count.to_string().as_str(), "-W", self.timeout.to_string().as_str(), host.ip_address.to_string().as_str()
            ]);
        }
        else {
            return Err(LkError::unsupported_platform());
        }

        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.is_success() {
            if let Some(last_line) = response.message.lines().last() {
                let average_latency = last_line.split('/').nth(4).ok_or(LkError::unexpected())?;
                Ok(DataPoint::value_with_level(average_latency.to_string(), Criticality::Normal))
            }
            else {
                Err(LkError::unexpected().to_string())
            }
        }
        else {
            Ok(DataPoint::value_with_level(response.message, Criticality::Critical))
        }
    }
}