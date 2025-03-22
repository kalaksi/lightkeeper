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

#[monitoring_module(
    name="filesystem",
    version="0.0.1",
    description="Shows filesystem usage in a progress bar.",
    settings={
        ignored_filesystems => "Comma-separated list of filesystems to ignore. Default: /run,/dev,/dev/shm,/sys/fs/cgroup",
        warning_threshold => "Warning threshold in percent. Default: 80",
        error_threshold => "Error threshold in percent. Default: 90",
        critical_threshold => "Critical threshold in percent. Default: 95",
    }
)]
pub struct Filesystem {
    ignored_filesystems: Vec<String>,
    threshold_critical: f64,
    threshold_error: f64,
    threshold_warning: f64,
}

impl Module for Filesystem {
    fn new(settings: &HashMap<String, String>) -> Self {
        Filesystem {
            ignored_filesystems: vec![
                String::from("/run"),
                String::from("/dev"),
                String::from("/dev/shm"),
                String::from("/sys/fs"),
                String::from("/var/lib/docker"),
            ],
            threshold_critical: settings.get("critical_threshold").unwrap_or(&String::from("95")).parse().unwrap(),
            threshold_error: settings.get("error_threshold").unwrap_or(&String::from("90")).parse().unwrap(),
            threshold_warning: settings.get("warning_threshold").unwrap_or(&String::from("80")).parse().unwrap(),
        }
    }
}

impl MonitoringModule for Filesystem {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::ProgressBar,
            display_text: String::from("Filesystem usage"),
            category: String::from("storage"),
            unit: String::from("%"),
            use_multivalue: true,
            use_with_charts: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> Result<String, LkError> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(String::from("df -hPT"))
        }
        else {
            Err(LkError::unsupported_platform())
        }
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        // NOTE: Non-zero exit code means that at least SOME errors were encountered, but might still have valid data.
        let mut result = DataPoint::empty();

        // First line contains headers
        let lines = response.message.lines().skip(1);
        for line in lines {
            let parts = line.split_whitespace().collect::<Vec<&str>>();
            if parts.len() < 7 {
                ::log::debug!("Invalid line in response: {}", line);
                continue;
            }

            // let _source = parts[0].to_string();
            let fs_type = parts[1].to_string();
            let size_h = parts[2].to_string();
            let used_h = parts[3].to_string();
            // let _available_h = parts[4].to_string();

            let mut used_percent = parts[5].to_string();
            // Remove percent symbol from the end.
            used_percent.pop();
            let used_percent_float = used_percent.parse::<f64>().unwrap();

            let mountpoint = parts[6].to_string();

            if self.ignored_filesystems.iter().any(|item| mountpoint.starts_with(item)) {
                continue;
            }

            let mut data_point = DataPoint::labeled_value(mountpoint.clone(), format!("{} %", used_percent));
            data_point.value_int = used_percent_float as i64;
            data_point.criticality = if used_percent_float >= self.threshold_critical {
                Criticality::Critical
            }
            else if used_percent_float >= self.threshold_error {
                Criticality::Error
            }
            else if used_percent_float >= self.threshold_warning {
                Criticality::Warning
            }
            else {
                Criticality::Normal
            };
            data_point.description = format!("{} | {} / {} used", fs_type, used_h, size_h);
            data_point.command_params.push(mountpoint);
            result.multivalue.push(data_point);
        }

        result.update_criticality_from_children();

        Ok(result)
    }
}