
use std::collections::HashMap;
use crate::enums::Criticality;
use crate::error::LkError;
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
    name="storage-lvm-logical-volume",
    version="0.0.1",
    description="Provides information about LVM logical volumes.",
    settings={
        snapshot_warning_threshold => "Snapshot utilization warning threshold in percent. Default: 80",
        snapshot_error_threshold => "Snapshot utilization error threshold in percent. Default: 90",
    }
)]
pub struct LogicalVolume {
    threshold_warning: f32,
    threshold_error: f32,
}

impl Module for LogicalVolume {
    fn new(settings: &HashMap<String, String>) -> Self {
        LogicalVolume {
            threshold_warning: settings.get("warning_threshold").unwrap_or(&String::from("80")).parse().unwrap(),
            threshold_error: settings.get("error_threshold").unwrap_or(&String::from("90")).parse().unwrap(),
        }
    }
}

impl MonitoringModule for LogicalVolume {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: String::from("Logical Volumes"),
            category: String::from("storage"),
            use_multivalue: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        let mut command = ShellCommand::new();
        command.use_sudo = host.settings.contains(&HostSetting::UseSudo);

        if host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
           host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
           host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
           host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") {
            command.arguments(vec![
                "lvs", "--separator", "|", "--options", "lv_path,lv_name,vg_name,lv_size,lv_attr,sync_percent,raid_mismatch_count,snap_percent", "--units", "H"
            ]);

            Ok(command.to_string())
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.message.is_empty() && response.return_code == 0 {
            return Ok(DataPoint::empty());
        }
        // Command does not exist.
        else if response.message.is_empty() && response.return_code == 127 {
            return Ok(DataPoint::empty());
        }

        let mut result = DataPoint::empty();

        let lines = response.message.lines().skip(1);
        for line in lines {
            let mut parts = line.split("|");
            let lv_path = parts.next().unwrap().trim().to_string();
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

            match lv_attr.chars().nth(8).unwrap() {
                'p' => {
                    data_point.tags.push(String::from("Partial"));
                    data_point.criticality = crate::enums::Criticality::Error;
                    data_point.value = String::from("Unknown % sync");
                },
                'r' => {
                    data_point.criticality = crate::enums::Criticality::Warning;
                    data_point.value = String::from("Refresh needed");
                }
                _ => {}
            }

            if !raid_mismatch_count.is_empty() && raid_mismatch_count != "0" {
                data_point.value = format!("{} mismatches", raid_mismatch_count);
                if data_point.criticality < Criticality::Warning {
                    data_point.criticality = Criticality::Warning;
                }
            }
            else if !sync_percent.is_empty() && sync_percent != "100.00" {
                data_point.value = format!("{}% sync", sync_percent);
                if data_point.criticality < Criticality::Warning {
                    data_point.criticality = Criticality::Warning;
                }
            }
            else if !snapshot_full_percent.is_empty() {
                data_point.value = format!("{}% full", snapshot_full_percent);
                if let Ok(fullness) = snapshot_full_percent.parse::<f32>() {
                    if fullness > self.threshold_warning && data_point.criticality < Criticality::Warning {
                        data_point.criticality = Criticality::Warning;
                    }
                    if fullness > self.threshold_error && data_point.criticality < Criticality::Error {
                        data_point.criticality = Criticality::Error;
                    }
                }
            }

            data_point.command_params = vec![lv_path, vg_name, lv_name, lv_size];
            result.multivalue.push(data_point);
        }

        if let Some(most_critical) = result.multivalue.iter().max_by_key(|point| point.criticality) {
            result.criticality = most_critical.criticality;
        }

        Ok(result)
    }
}