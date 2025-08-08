/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;
use crate::enums::Criticality;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::module::platform_info;
use crate::{
    Host,
    frontend,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="ram",
    version="0.0.1",
    description="Provides RAM usage information.",
    settings={
        warning_threshold => "Warning threshold in percent. Default: 70",
        error_threshold => "Error threshold in percent. Default: 80",
        critical_threshold => "Critical threshold in percent. Default: 90",
    }
)]
pub struct Ram {
    threshold_critical: f32,
    threshold_error: f32,
    threshold_warning: f32,
}

impl Module for Ram {
    fn new(settings: &HashMap<String, String>) -> Self {
        Ram {
            threshold_critical: settings.get("critical_threshold").and_then(|value| value.parse().ok()).unwrap_or(90.0),
            threshold_error: settings.get("error_threshold").and_then(|value| value.parse().ok()).unwrap_or(80.0),
            threshold_warning: settings.get("warning_threshold").and_then(|value| value.parse().ok()).unwrap_or(70.0),
        }
    }
}

impl MonitoringModule for Ram {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::ProgressBar,
            display_text: String::from("RAM usage"),
            category: String::from("host"),
            unit: String::from("%"),
            use_with_charts: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, LkError> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("free -m"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        if response.is_error() {
            return Err(response.message);
        }

        let line = response.message.lines().filter(|line| line.contains("Mem:")).collect::<Vec<&str>>();
        let parts = line[0].split_whitespace().collect::<Vec<&str>>();

        let total = parts[1].parse::<u32>().map_err(|_| String::from("Unsupported platform"))?;
        // used
        // free
        // shared
        // cache
        let available = parts[6].parse::<u32>().map_err(|_| String::from("Unsupported platform"))?;

        let usage_percent = (total - available) as f32 / total as f32 * 100.0;

        let criticality = if usage_percent >= self.threshold_critical {
            Criticality::Critical
        }
        else if usage_percent >= self.threshold_error {
            Criticality::Error
        }
        else if usage_percent >= self.threshold_warning {
            Criticality::Warning
        }
        else {
            Criticality::Normal
        };

        let value = format!("{:.0} % of {} M", usage_percent, total);
        let mut data_point = DataPoint::value_with_level(value, criticality);
        data_point.value_float = usage_percent;
        Ok(data_point)
    }
}