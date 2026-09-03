/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */


use std::collections::HashMap;

use crate::enums::Criticality;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::module::platform_info;
use crate::utils::ShellCommand;
use crate::{
    frontend,
    Host,
    HostSetting,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="haproxy",
    version="0.0.1",
    description="Provides HAProxy frontend, backend and server status from the stats socket.",
    uses_sudo=true,
    settings={
        socket_file => "HAProxy admin/runtime stats socket file. Default: /run/haproxy/admin.sock",
        included_proxies => "Comma-separated list of proxy (pxname) prefixes to include. Default: empty (all).",
        excluded_proxies => "Comma-separated list of proxy (pxname) prefixes to exclude. Default: empty",
        include_servers => "Include individual server rows in addition to FRONTEND/BACKEND rows. Default: true",
    }
)]
pub struct Haproxy {
    socket_file: String,
    included_proxies: Vec<String>,
    excluded_proxies: Vec<String>,
    include_servers: bool,
}

impl Module for Haproxy {
    fn new(settings: &HashMap<String, String>) -> Self {
        Haproxy {
            socket_file: settings.get("socket_file")
                .cloned()
                .unwrap_or_else(|| String::from("/run/haproxy/admin.sock")),
            included_proxies: settings.get("included_proxies").unwrap_or(&String::from(""))
                .split(',')
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string())
                .collect(),
            excluded_proxies: settings.get("excluded_proxies").unwrap_or(&String::from(""))
                .split(',')
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string())
                .collect(),
            include_servers: settings.get("include_servers")
                .map(|value| value != "false")
                .unwrap_or(true),
        }
    }
}

impl MonitoringModule for Haproxy {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("HAProxy"),
            category: String::from("haproxy"),
            use_multivalue: true,
            use_with_charts: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, LkError> {
        if host.platform.os != platform_info::OperatingSystem::Linux {
            return Err(LkError::unsupported_platform());
        }

        if !is_safe_socket_file(&self.socket_file) {
            return Err(LkError::other("Invalid HAProxy socket_file"));
        }

        // Stats socket speaks the line protocol (show stat), not HTTP.
        // Only nc needs elevated privileges to open the socket.
        let mut command = ShellCommand::new();
        command.arguments(vec!["echo", "show stat"]);
        if host.settings.contains(&HostSetting::UseSudo) {
            command.pipe_to(vec!["sudo", "nc", "-U", self.socket_file.as_str()]);
        }
        else {
            command.pipe_to(vec!["nc", "-U", self.socket_file.as_str()]);
        }
        Ok(command.to_string())
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _parent_result: DataPoint) -> Result<DataPoint, String> {
        if response.is_command_not_found() {
            return Ok(DataPoint::value_with_level(
                String::from("nc or HAProxy stats socket unavailable"),
                Criticality::NotAvailable,
            ));
        }
        if response.is_error() {
            return Ok(DataPoint::value_with_level(response.message, Criticality::Error));
        }
        if response.message.trim().is_empty() {
            return Ok(DataPoint::value_with_level(
                String::from("Empty response from HAProxy stats socket"),
                Criticality::Error,
            ));
        }

        let mut lines = response.message.lines();
        let header_line = lines.next().unwrap_or_default().trim_start_matches('#').trim();
        if !header_line.starts_with("pxname") {
            return Ok(DataPoint::value_with_level(
                String::from("Unexpected HAProxy show stat response"),
                Criticality::Error,
            ));
        }

        let headers: Vec<&str> = header_line.split(',').collect();
        let index_of = |name: &str| headers.iter().position(|header| *header == name);

        let pxname_i = index_of("pxname").ok_or_else(|| String::from("Missing pxname column"))?;
        let svname_i = index_of("svname").ok_or_else(|| String::from("Missing svname column"))?;
        let status_i = index_of("status").ok_or_else(|| String::from("Missing status column"))?;
        let scur_i = index_of("scur");
        let slim_i = index_of("slim");
        let qcur_i = index_of("qcur");
        let type_i = index_of("type");
        let mode_i = index_of("mode");
        let check_status_i = index_of("check_status");
        let last_chk_i = index_of("last_chk");
        let addr_i = index_of("addr");

        let mut result = DataPoint::empty();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split(',').collect();
            let pxname = fields.get(pxname_i).copied().unwrap_or_default();
            let svname = fields.get(svname_i).copied().unwrap_or_default();
            if pxname.is_empty() || svname.is_empty() {
                continue;
            }

            if !self.included_proxies.is_empty()
                && !self.included_proxies.iter().any(|prefix| pxname.starts_with(prefix))
            {
                continue;
            }
            if self.excluded_proxies.iter().any(|prefix| pxname.starts_with(prefix)) {
                continue;
            }

            let row_type = type_i
                .and_then(|i| fields.get(i))
                .and_then(|value| value.parse::<u8>().ok())
                .unwrap_or_else(|| infer_type(svname));

            // 0=frontend, 1=backend, 2=server, 3=listener/socket
            if row_type == 2 && !self.include_servers {
                continue;
            }

            let status = fields.get(status_i).copied().unwrap_or_default();
            let scur = scur_i
                .and_then(|i| fields.get(i))
                .and_then(|value| value.parse::<f32>().ok())
                .unwrap_or(0.0);
            let slim = slim_i
                .and_then(|i| fields.get(i))
                .and_then(|value| value.parse::<f32>().ok());
            let qcur = qcur_i
                .and_then(|i| fields.get(i))
                .and_then(|value| value.parse::<u32>().ok())
                .unwrap_or(0);

            let mut point = DataPoint::labeled_value(format!("{}/{}", pxname, svname), format_status(status));
            point.criticality = criticality_for_status(status);
            point.command_params = vec![pxname.to_string(), svname.to_string()];

            point.tags.push(match row_type {
                0 => String::from("Frontend"),
                1 => String::from("Backend"),
                2 => String::from("Server"),
                _ => String::from("Listener"),
            });

            let mut description_parts = Vec::new();
            if let Some(mode) = mode_i.and_then(|i| fields.get(i)).filter(|mode| !mode.is_empty()) {
                description_parts.push(mode.to_uppercase());
            }

            if let Some(limit) = slim.filter(|limit| *limit > 0.0) {
                point.value_float = scur / limit * 100.0;
                description_parts.push(format!("sessions {:.0}/{:.0}", scur, limit));
            }
            else {
                point.value_float = scur;
                description_parts.push(format!("sessions {:.0}", scur));
            }

            if qcur > 0 {
                description_parts.push(format!("queue {}", qcur));
            }

            if let Some(check_status) = check_status_i
                .and_then(|i| fields.get(i))
                .filter(|value| !value.is_empty())
            {
                let last_chk = last_chk_i
                    .and_then(|i| fields.get(i))
                    .filter(|value| !value.is_empty())
                    .unwrap_or(&"");
                if last_chk.is_empty() {
                    description_parts.push(check_status.to_string());
                }
                else {
                    description_parts.push(format!("{} ({})", check_status, last_chk));
                }
            }

            if let Some(addr) = addr_i.and_then(|i| fields.get(i)).filter(|value| !value.is_empty()) {
                description_parts.push(addr.to_string());
            }

            point.description = description_parts.join(" | ");
            result.multivalue.push(point);
        }

        if result.multivalue.is_empty() {
            return Ok(DataPoint::value_with_level(
                String::from("No HAProxy proxies found"),
                Criticality::Normal,
            ));
        }

        result.update_criticality_from_children();
        Ok(result)
    }
}

fn format_status(status: &str) -> String {
    let mut chars = status.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        None => String::new(),
    }
}

fn infer_type(svname: &str) -> u8 {
    match svname {
        "FRONTEND" => 0,
        "BACKEND" => 1,
        _ => 2,
    }
}

fn criticality_for_status(status: &str) -> Criticality {
    let status_upper = status.to_uppercase();
    if status_upper.starts_with("DOWN") {
        Criticality::Critical
    }
    else if status_upper.starts_with("MAINT")
        || status_upper.starts_with("DRAIN")
        || status_upper == "NOLB"
        || status_upper == "STOPPING"
    {
        Criticality::Warning
    }
    else {
        // UP, OPEN, no check, etc.
        Criticality::Normal
    }
}

fn is_safe_socket_file(path: &str) -> bool {
    !path.is_empty()
        && path.starts_with('/')
        && path.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '/' || c == '.' || c == '_' || c == '-'
        })
}
