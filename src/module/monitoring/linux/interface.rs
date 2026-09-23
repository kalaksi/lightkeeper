/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use crate::error::LkError;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend, enums,
};

use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="interface",
    version="0.0.1",
    description="Provides information about network interfaces.",
    settings={
        ignored_interfaces => "Comma-separated list of interface names or name prefixes to ignore. Default: empty."
    }
)]
pub struct Interface {
    ignored_interfaces: Vec<String>,
}

impl Module for Interface {
    fn new(settings: &HashMap<String, String>) -> Self {
        Interface {
            ignored_interfaces: settings.get("ignored_interfaces").unwrap_or(&String::from(""))
                                        .split(',')
                                        .filter(|value| !value.is_empty())
                                        .map(|value| value.to_string())
                                        .collect(),
        }
    }
}

impl MonitoringModule for Interface {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Interfaces"),
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
        if host.platform.os_flavor == platform_info::Flavor::CentOS ||
           host.platform.os_flavor == platform_info::Flavor::RedHat {
            Ok(String::from("/sbin/ip -j addr show"))
        }
        else if host.platform.os_flavor == platform_info::Flavor::Alpine {
            // BusyBox ip has no -j; parse plain text instead.
            Ok(String::from("ip addr show"))
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("ip -j addr show"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.is_error() {
            return Err(response.message);
        }

        let mut result = DataPoint::empty();

        let interfaces = if host.platform.os_flavor == platform_info::Flavor::Alpine {
            parse_ip_addr_text(&response.message)?
        }
        else {
            serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?
        };

        for interface in interfaces.iter() {
            if self.ignored_interfaces.iter().any(|item| interface.ifname.starts_with(item)) {
                continue;
            }

            let mut data_point = DataPoint::labeled_value(
                interface.ifname.clone(),
                interface.operstate.clone().to_lowercase(),
            );
            if let Some(address) = &interface.address {
                data_point.description = format!("{}", address);
            }

            if interface.flags.contains(&String::from("NO-CARRIER")) {
                data_point.tags.push(String::from("NO-CARRIER"));
            }

            if interface.flags.contains(&String::from("POINTOPOINT")) {
                data_point.tags.push(String::from("POINTOPOINT"));
            }

            if interface.operstate == "DOWN" {
                data_point.criticality = enums::Criticality::Error;
            }
            else if interface.operstate == "UP" {
                data_point.criticality = enums::Criticality::Normal;
            }
            else {
                data_point.criticality = enums::Criticality::Ignore;
            }

            for address in interface.addr_info.iter() {
                let address_with_prefix = format!("{}/{}", address.local, address.prefixlen);
                let address_datapoint = DataPoint::labeled_value(address_with_prefix, String::from(""));
                data_point.multivalue.push(address_datapoint);
            }

            result.multivalue.push(data_point);
        }

        result.update_criticality_from_children();
        Ok(result)
    }
}

fn parse_ip_addr_text(message: &str) -> Result<Vec<InterfaceDetails>, String> {
    let mut interfaces = Vec::new();
    let mut current: Option<InterfaceDetails> = None;

    for line in message.lines() {
        let trimmed = line.trim_start();

        // Header: "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 ... state UP ..."
        if let Some(index_end) = line.find(": ") {
            if line[..index_end].chars().all(|c| c.is_ascii_digit()) {
                let after_index = &line[index_end + 2..];
                if let Some(name_end) = after_index.find(": ") {
                    let ifname = after_index[..name_end].to_string();
                    let after_name = &after_index[name_end + 2..];
                    if after_name.starts_with('<') {
                        if let Some(interface) = current.take() {
                            interfaces.push(interface);
                        }

                        let flags_end = after_name.find('>').ok_or("Invalid interface flags")?;
                        let flags = after_name[1..flags_end]
                            .split(',')
                            .filter(|flag| !flag.is_empty())
                            .map(|flag| flag.to_string())
                            .collect();
                        let operstate = after_name[flags_end + 1..]
                            .split_whitespace()
                            .skip_while(|word| *word != "state")
                            .nth(1)
                            .unwrap_or("UNKNOWN")
                            .to_string();

                        current = Some(InterfaceDetails {
                            ifname,
                            flags,
                            operstate,
                            link_type: String::new(),
                            address: None,
                            addr_info: Vec::new(),
                        });
                        continue;
                    }
                }
            }
        }

        let Some(interface) = current.as_mut() else {
            continue;
        };

        if trimmed.starts_with("link/") {
            let mut parts = trimmed.split_whitespace();
            let link_type = parts.next().unwrap_or_default().trim_start_matches("link/");
            interface.link_type = link_type.to_string();
            if let Some(mac) = parts.next() {
                if mac.contains(':') {
                    interface.address = Some(mac.to_string());
                }
            }
        }
        else if trimmed.starts_with("inet ") || trimmed.starts_with("inet6 ") {
            let mut parts = trimmed.split_whitespace();
            let family = parts.next().unwrap_or_default().to_string();
            let Some(addr_prefix) = parts.next() else {
                continue;
            };
            let Some((local, prefix)) = addr_prefix.split_once('/') else {
                continue;
            };
            let Ok(prefixlen) = prefix.parse::<u8>() else {
                continue;
            };
            interface.addr_info.push(InterfaceAddress {
                family,
                local: local.to_string(),
                prefixlen,
            });
        }
    }

    if let Some(interface) = current {
        interfaces.push(interface);
    }

    Ok(interfaces)
}

#[derive(Deserialize)]
pub struct InterfaceDetails {
    pub ifname: String,
    pub flags: Vec<String>,
    pub operstate: String,
    pub link_type: String,
    pub address: Option<String>,
    pub addr_info: Vec<InterfaceAddress>,
}

#[derive(Deserialize)]
pub struct InterfaceAddress {
    pub family: String,
    pub local: String,
    pub prefixlen: u8,
}
