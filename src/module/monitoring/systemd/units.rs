/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use serde::Deserialize;
use serde_json;

use crate::enums;
use crate::error::LkError;
use crate::host::Host;
use crate::module::connection::ResponseMessage;
use crate::module::monitoring::DataPoint;
use crate::module::platform_info;
use crate::utils::ShellCommand;

pub fn list_units_command(host: &Host, user: bool) -> Result<String, LkError> {
    let mut command = ShellCommand::new();
    let mut args = vec!["busctl", "--no-pager"];
    if user {
        args.push("--user");
    }

    if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::AlmaLinux, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::Rocky, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::AmazonLinux, "2023")
    {
        args.extend([
            "call", "org.freedesktop.systemd1", "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager", "ListUnits",
        ]);
        command.arguments(args);
        Ok(command.to_string())
    }
    else if host.platform.os == platform_info::OperatingSystem::Linux {
        args.extend([
            "--json=short", "call", "org.freedesktop.systemd1", "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager", "ListUnits",
        ]);
        command.arguments(args);
        Ok(command.to_string())
    }
    else {
        Err(LkError::unsupported_platform())
    }
}

pub fn process_units_response(
    host: &Host,
    response: ResponseMessage,
    included_services: &[String],
    excluded_services: &[String],
    unavailable_message: &str,
) -> Result<DataPoint, String> {
    if response.is_command_not_found() {
        return Ok(DataPoint::value_with_level(unavailable_message.to_string(), enums::Criticality::NotAvailable));
    }
    if response.is_error() {
        if response.message.contains("Failed to connect to bus") {
            return Ok(DataPoint::value_with_level(unavailable_message.to_string(), enums::Criticality::NotAvailable));
        }
        return Err(response.message);
    }

    let all_units = parse_units(host, &response.message)?;
    let services = all_units.iter().filter(|unit| unit.id.ends_with(".service"));
    let mut allowed_services = services
        .filter(|unit| included_services.is_empty() || included_services.iter().any(|id| unit.id.starts_with(id)))
        .filter(|unit| excluded_services.is_empty() || !excluded_services.iter().any(|id| unit.id.starts_with(id)))
        .collect::<Vec<&UnitData>>();

    allowed_services.sort_by_key(|unit| &unit.id);

    let mut result = DataPoint::empty();
    result.multivalue = allowed_services.iter().map(|unit| {
        let mut point = DataPoint::labeled_value(unit.id.clone(), unit.sub_state.clone());
        point.description = unit.name.clone();

        if ["masked"].contains(&unit.load_state.as_str()) {
            point.tags.push(unit.load_state.clone());
        }

        match unit.sub_state.as_str() {
            "dead" => {
                if unit.active_state == "failed" {
                    point.criticality = enums::Criticality::Critical;
                    point.value = format!("{} ({})", unit.sub_state, unit.active_state);
                }
                else {
                    // Inactive/dead is normal for stopped services and successful oneshots/timers.
                    point.criticality = enums::Criticality::Normal;
                    if unit.active_state != "inactive" {
                        point.value = format!("{} ({})", unit.sub_state, unit.active_state);
                    }
                }
            },
            "exited" => {
                if unit.active_state == "active" {
                    point.criticality = enums::Criticality::Normal;
                    point.value = format!("{} ({})", unit.sub_state, unit.active_state);
                    point.tags.push(String::from("active"));
                }
                else {
                    point.criticality = enums::Criticality::Warning;
                }
            },
            "running" => {
                point.criticality = enums::Criticality::Normal;
                point.tags.push(String::from("active"));
            },
            _ => {
                point.criticality = enums::Criticality::Warning;
            }
        };

        point.command_params.push(unit.id.clone());
        point
    }).collect();

    result.update_criticality_from_children();
    Ok(result)
}

fn parse_units(host: &Host, message: &str) -> Result<Vec<UnitData>, String> {
    if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::AlmaLinux, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::Rocky, "8") ||
        host.platform.is_same_or_greater(platform_info::Flavor::AmazonLinux, "2023")
    {
        // First 2 words are like: a(ssssssouso) 253
        let actual_content = message.splitn(3, " ").last().unwrap_or("");
        let fields: Vec<&str> = actual_content[1..actual_content.len() - 1]
            .split("\" ")
            .map(|field| field.trim_matches('"'))
            .collect();

        Ok(fields.chunks_exact(9).map(|chunk| {
            UnitData {
                id: chunk[0].to_string(),
                name: chunk[1].to_string(),
                load_state: chunk[2].to_string(),
                active_state: chunk[3].to_string(),
                sub_state: chunk[4].to_string(),
                _follows: chunk[5].to_string(),
                _unit_path: chunk[6].to_string(),
                _job_id: chunk[7].split(" ").next().and_then(|value| value.parse::<u32>().ok()).unwrap_or(0),
                _job_type: chunk[7].split(" \"").nth(1).unwrap_or("").to_string(),
                _job_path: chunk[8].to_string(),
            }
        }).collect())
    }
    else if host.platform.os == platform_info::OperatingSystem::Linux {
        let mut response: DbusResponse = serde_json::from_str(message).map_err(|e| e.to_string())?;
        Ok(response.data.remove(0))
    }
    else {
        Ok(Vec::new())
    }
}

#[derive(Deserialize)]
struct DbusResponse {
    data: Vec<Vec<UnitData>>,
}

#[derive(Deserialize)]
struct UnitData {
    id: String,
    name: String,
    load_state: String,
    active_state: String,
    sub_state: String,
    _follows: String,
    _unit_path: String,
    _job_id: u32,
    _job_type: String,
    _job_path: String,
}
