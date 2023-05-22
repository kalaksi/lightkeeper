
use std::collections::HashMap;
use crate::enums::Criticality;
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

#[monitoring_module("lvm-volume", "0.0.1")]
pub struct Volume {
}

impl Module for Volume {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Volume {
        }
    }
}

impl MonitoringModule for Volume {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("LVM logical volumes"),
            category: String::from("storage"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> String {
        let mut command = ShellCommand::new();

        if host.platform.os == platform_info::OperatingSystem::Linux {
            if host.platform.version_is_newer_than(platform_info::Flavor::Debian, "8") &&
               host.platform.version_is_older_than(platform_info::Flavor::Debian, "11") {
                command.arguments(vec![
                    "lvs", "--separator", "|", "--options", "lv_path,lv_name,vg_name,lv_size,lv_attr,sync_percent,raid_mismatch_count,snap_percent"
                ]);
            }

            command.use_sudo = host.settings.contains(&HostSetting::UseSudo);
        }

        command.to_string()
    }

    fn process_response(&self, host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if host.platform.version_is_newer_than(platform_info::Flavor::Debian, "8") &&
           host.platform.version_is_older_than(platform_info::Flavor::Debian, "11") {

            if response.message.is_empty() && response.return_code == 0 {
                return Ok(DataPoint::empty());
            }

            let mut result = DataPoint::empty();

            let lines = response.message.split('\n').skip(1);
            for line in lines {
                let mut parts = line.split("|");
                let _lv_path = parts.next().unwrap().to_string();
                let lv_name = parts.next().unwrap().to_string();
                let vg_name = parts.next().unwrap().to_string();
                let lv_size = parts.next().unwrap().to_string();
                let lv_attr = parts.next().unwrap().to_string();
                let sync_percent = parts.next().unwrap().to_string();
                let raid_mismatch_count = parts.next().unwrap().to_string();
                let snapshot_full_percent = parts.next().unwrap().to_string();


                let mut data_point = DataPoint::labeled_value(lv_name.clone(), String::from("OK"));
                data_point.description = format!("{} | size: {}", vg_name, lv_size);

                match lv_attr.chars().nth(0).unwrap() {
                    'r' => data_point.tags.push(String::from("RAID")),
                    'R' => data_point.tags.push(String::from("RAID")),
                    'm' => data_point.tags.push(String::from("Mirror")),
                    'M' => data_point.tags.push(String::from("Mirror")),
                    's' => data_point.tags.push(String::from("Snapshot")),
                    'p' => data_point.tags.push(String::from("pvmove")),
                    _ => {}
                }

                match lv_attr.chars().nth(1).unwrap() {
                    'r' => data_point.tags.push(String::from("Read-only")),
                    _ => {}
                }

                if lv_attr.chars().nth(5).unwrap() == 'o' {
                    data_point.description = format!("{} | Open", data_point.description);
                }
                else if lv_attr.chars().nth(4).unwrap() == 'a' {
                    data_point.description = format!("{} | Active", data_point.description);
                }

                if lv_attr.chars().nth(8).unwrap() == 'p' {
                    data_point.tags.push(String::from("Partial"));
                    data_point.criticality = crate::enums::Criticality::Error;
                    data_point.value = String::from("Unknown % sync");
                }

                if !raid_mismatch_count.is_empty() && raid_mismatch_count != "0" {
                    data_point.value = format!("{} mismatches", raid_mismatch_count);
                    if data_point.criticality < Criticality::Warning {
                        data_point.criticality = Criticality::Warning;
                    }
                }
                else

                if !sync_percent.is_empty() && sync_percent != "100.00" {
                    data_point.value = format!("{}% sync", sync_percent);
                    if data_point.criticality < Criticality::Warning {
                        data_point.criticality = Criticality::Warning;
                    }
                }
                else if !snapshot_full_percent.is_empty() {
                    data_point.value = format!("{}% full", snapshot_full_percent);
                    let fullness = snapshot_full_percent.parse::<f32>().unwrap();

                    if fullness > 50.0 && data_point.criticality < Criticality::Warning {
                        data_point.criticality = Criticality::Warning;
                    }
                    if fullness > 75.0 && data_point.criticality < Criticality::Error {
                        data_point.criticality = Criticality::Error;
                    }
                }
                result.multivalue.push(data_point);
            }

            Ok(result)
        }
        else {
            self.error_unsupported()
        }
    }
}