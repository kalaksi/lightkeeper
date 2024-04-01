
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};

use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;
use crate::utils::ShellCommand;
use crate::host::HostSetting;

#[monitoring_module(
    name="storage-lvm-volume-group",
    version="0.0.1",
    description="Provides information about LVM volume groups.",
)]
pub struct VolumeGroup {
}

impl Module for VolumeGroup {
    fn new(_settings: &HashMap<String, String>) -> Self {
        VolumeGroup {
        }
    }
}

impl MonitoringModule for VolumeGroup {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Volume Groups"),
            category: String::from("storage"),
            use_multivalue: true,
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, String> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {
            command.arguments(vec![ "vgs", "--separator", "|", "--options", "vg_name,vg_attr,vg_size,vg_free", "--units", "H" ]);
            Ok(command.to_string())
        }
        else {
            Err(String::from("Unsupported platform"))
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.message.is_empty() && response.return_code == 0 {
            return Ok(DataPoint::empty());
        }

        let mut result = DataPoint::empty();

        let lines = response.message.lines().skip(1);
        for line in lines {
            let mut parts = line.split("|");
            let vg_name = parts.next().unwrap().trim_start().to_string();
            let vg_attr = parts.next().unwrap().to_string();
            let vg_size = parts.next().unwrap().to_string();
            let vg_free = parts.next().unwrap().to_string();

            let mut data_point = DataPoint::labeled_value(vg_name.clone(), String::from("OK"));
            data_point.description = format!("free: {} / {}", vg_free, vg_size);

            match vg_attr.chars().nth(0).unwrap() {
                'r' => data_point.tags.push(String::from("Read-only")),
                _ => {}
            }

            if vg_attr.chars().nth(5).unwrap() == 'p' {
                data_point.criticality = crate::enums::Criticality::Error;
                data_point.value = String::from("Partial");
            }

            data_point.command_params = vec![vg_name];
            result.multivalue.push(data_point);
        }

        Ok(result)
    }
}