
use std::collections::HashMap;

use crate::module::connection::ResponseMessage;
use crate::Host;
use crate::utils::{VersionNumber, string_manipulation};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("_internal-platform-info-ssh", "0.0.1")]
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
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_messages(&self, _host: Host, _result: DataPoint) -> Result<Vec<String>, String> {
        Ok(vec![
            String::from("cat /etc/os-release"),
            String::from("uname -m"),
        ])
    }

    fn process_responses(&self, _host: Host, response: Vec<ResponseMessage>, _result: DataPoint) -> Result<DataPoint, String> {
        let mut platform = PlatformInfo::default();
        platform.os = platform_info::OperatingSystem::Linux;

        (platform.os_flavor, platform.os_version) = parse_os_release(&response[0].message);
        platform.architecture = parse_architecture(&response[1].message);

        // Special kind of datapoint for internal use.
        let mut datapoint = DataPoint::new(String::from("_platform_info"));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os"), platform.os.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os_version"), platform.os_version.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os_flavor"), platform.os_flavor.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("architecture"), platform.architecture.to_string()));
        Ok(datapoint)
    }
}

fn parse_os_release(message: &String) -> (platform_info::Flavor, VersionNumber) {
    let mut flavor = platform_info::Flavor::default();
    let mut version = VersionNumber::default();

    let lines = message.lines();
    for line in lines {
        let mut parts = line.split('=');
        let key = parts.next().unwrap_or_default();
        let value = string_manipulation::remove_quotes(&parts.next().unwrap_or_default());

        match key {
            "ID" => {
                match value.as_str() {
                    "debian" => flavor = platform_info::Flavor::Debian,
                    "centos" => flavor = platform_info::Flavor::CentOS,
                    _ => ()
                }
            },
            "VERSION_ID" => version = VersionNumber::from_string(&value.to_string()),
            _ => ()
        }
    }

    (flavor, version)
}

fn parse_architecture(message: &String) -> platform_info::Architecture {
    if message.contains("x86_64") {
        platform_info::Architecture::X86_64
    }
    else {
        platform_info::Architecture::Unknown
    }
}