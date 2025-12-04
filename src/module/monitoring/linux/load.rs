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
    name="load",
    version="0.0.1",
    description="Provides information about average load (using uptime-command).",
    settings={
        value_max => "Maximum value for the load average. Affects charts. Default: 20",
    }
)]
pub struct Load {
    value_max: f64,
}

impl Module for Load {
    fn new(settings: &HashMap<String, String>) -> Self {
        Load {
            value_max: settings.get("value_max").and_then(|value| value.parse().ok()).unwrap_or(20.0),
        }
    }
}

impl MonitoringModule for Load {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Loads"),
            category: String::from("host"),
            use_with_charts: true,
            value_max: self.value_max,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, LkError> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("uptime"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        if response.is_error() {
            return Err(response.message);
        }

        let parts = response.message.split("load average: ").collect::<Vec<&str>>();
        let mut data_point = DataPoint::new(parts[1].to_string());

        let loads = parts[1].split(", ").collect::<Vec<&str>>();
        if loads.len() == 3 {
            let load_1 = loads[0].replace(",", ".").parse::<f32>().unwrap_or(0.0);
            data_point.value_float = load_1;
        }

        Ok(data_point)
    }
}