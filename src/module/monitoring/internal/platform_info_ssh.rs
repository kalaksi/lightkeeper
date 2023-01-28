
use std::collections::HashMap;

use crate::module::connection::ResponseMessage;
use crate::Host;
use crate::utils::VersionNumber;
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

    fn get_connector_message(&self, _host: Host) -> String {
        String::from("cat /proc/version")
    }

    fn process_response(&self, _host: Host, response: ResponseMessage) -> Result<DataPoint, String> {
        let mut platform = PlatformInfo::default();
        platform.os = platform_info::OperatingSystem::Linux;

        if let Some(index) = response.message.find("(Debian") {
            platform.os_flavor = platform_info::Flavor::Debian;

            if let Some(end_index) = response.message[index..].find(")") {
                let version_index = index + "(Debian ".chars().count();
                platform.os_version = VersionNumber::from_string(&response.message[version_index..(index + end_index)].to_string());
            }
        }
        else if let Some(index) = response.message.find("(Ubuntu") {
            platform.os_flavor = platform_info::Flavor::Ubuntu;

            if let Some(end_index) = response.message[index..].find(")") {
                let version_index = index + "(Ubuntu ".chars().count();
                platform.os_version = VersionNumber::from_string(&response.message[version_index..(index + end_index)].to_string());
            }
        }
        else if let Some(index) = response.message.find("(Red Hat") {
            platform.os_flavor = platform_info::Flavor::RedHat;

            if let Some(end_index) = response.message[index..].find(")") {
                let version_index = index + "(Red Hat ".chars().count();
                platform.os_version = VersionNumber::from_string(&response.message[version_index..(index + end_index)].to_string());
            }
        }
        else {
            platform.os_flavor = platform_info::Flavor::Unknown;
        }

        // Special kind of datapoint for internal use.
        let mut datapoint = DataPoint::new(String::from("_platform_info"));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os"), platform.os.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os_version"), platform.os_version.to_string()));
        datapoint.multivalue.push(DataPoint::labeled_value(String::from("os_flavor"), platform.os_flavor.to_string()));
        Ok(datapoint)
    }
}