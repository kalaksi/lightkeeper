use serde_derive::Deserialize;
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
        ignored_interfaces => "Comma-separated list of interface names to ignore. Default: empty."
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
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        if host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "8") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "8") {
            Ok(String::from("/sbin/ip -j addr show"))
        }
        else if host.platform.os_flavor == platform_info::Flavor::CentOS ||
                host.platform.os_flavor == platform_info::Flavor::RedHat {
            Err(LkError::unsupported_platform())
        }
        else if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("ip -j addr show"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();

        let mut interfaces: Vec<InterfaceDetails> = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;

        for interface in interfaces.iter_mut() {
            if self.ignored_interfaces.iter().any(|item| interface.ifname.starts_with(item)) {
                continue;
            }

            let mut data_point = DataPoint::labeled_value(interface.ifname.clone(), interface.operstate.clone().to_lowercase());
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
