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
use crate::module::monitoring::systemd::units;

#[monitoring_module(
    name="systemd-service",
    version="0.0.1",
    description="Provides information about SystemD services.",
    settings={
      included_services => "Comma-separated list of services to include. Default: empty (which means all).",
      excluded_services => "Comma-separated list of services to exclude. Default: empty"
    }
)]
pub struct Service {
    included_services: Vec<String>,
    excluded_services: Vec<String>,
}

impl Module for Service {
    fn new(settings: &HashMap<String, String>) -> Self {
        Service {
            included_services: settings.get("included_services").unwrap_or(&String::from(""))
                                       .split(',')
                                       .filter(|value| !value.is_empty())
                                       .map(|value| value.to_string())
                                       .collect(),
            excluded_services: settings.get("excluded_services").unwrap_or(&String::from(""))
                                       .split(',')
                                       .filter(|value| !value.is_empty())
                                       .map(|value| value.to_string())
                                       .collect(),
        }
    }
}

impl MonitoringModule for Service {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Services"),
            category: String::from("systemd"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        units::list_units_command(&host, false)
    }

    fn process_response(&self, host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        units::process_units_response(
            &host,
            response,
            &self.included_services,
            &self.excluded_services,
            "SystemD not available",
        )
    }
}
