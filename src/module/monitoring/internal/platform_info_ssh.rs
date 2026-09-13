/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;

use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::Host;
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="_internal-platform-info-ssh",
    version="0.0.1",
    description="Internal module to provide platform information with SSH.",
)]
pub struct PlatformInfoSsh {
}

impl Module for PlatformInfoSsh {
    fn new(_settings: &HashMap<String, String>) -> Self {
        PlatformInfoSsh {
        }
    }
}

impl MonitoringModule for PlatformInfoSsh {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_messages(&self, _host: Host, _result: DataPoint) -> Result<Vec<String>, LkError> {
        Ok(vec![
            String::from("cat /etc/os-release"),
            String::from("uname -m"),
        ])
    }

    fn process_responses(&self, host: Host, response: Vec<ResponseMessage>, _result: DataPoint) -> Result<DataPoint, String> {
        let os_release = response.get(0).map(|response| response.message.as_str()).unwrap_or("");
        let uname_machine = response.get(1).map(|response| response.message.as_str()).unwrap_or("");
        if os_release.is_empty() {
            return Err(String::from("No response for OS release"));
        }
        if uname_machine.is_empty() {
            return Err(String::from("No response for architecture"));
        }

        let platform = PlatformInfo::from_linux_probe(os_release, uname_machine);

        // Special kind of datapoint for internal use.
        // TODO: separate module type?
        let mut datapoint = DataPoint::new(String::from("_platform_info"));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os"), platform.os.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os_version"), platform.os_version.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os_flavor"), platform.os_flavor.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("architecture"), platform.architecture.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os_variant_id"), platform.os_variant_id.clone()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("ip_address"), host.ip_address.to_string()));
        Ok(datapoint)
    }
}
