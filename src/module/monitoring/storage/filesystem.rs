use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("filesystem", "0.0.1")]
pub struct Filesystem {
    ignored_filesystems: Vec<String>,
}

impl Module for Filesystem {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Filesystem {
            ignored_filesystems: vec![
                String::from("/run"),
                String::from("/dev/shm"),
                String::from("/sys/fs/cgroup"),
            ]
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
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host, _result: DataPoint) -> String {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            String::from("df -hPT")
        }
        else {
            String::new()
        }
    }

    fn process_response(&self, host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            let mut result = DataPoint::empty();

            // First line contains headers
            let mut lines = response.message.lines().skip(1);
            for line in lines {
                let mut parts = line.split_whitespace();
                let _source = parts.next().unwrap().to_string();
                let fs_type = parts.next().unwrap().to_string();
                let size_h = parts.next().unwrap().to_string();
                let used_h = parts.next().unwrap().to_string();
                let _available_h = parts.next().unwrap().to_string();
                let mut used_percent = parts.next().unwrap().to_string();
                let mountpoint = parts.next().unwrap().to_string();

                if self.ignored_filesystems.iter().any(|item| mountpoint.starts_with(item)) {
                    continue;
                }

                // Remove percent symbol from the end.
                used_percent.pop();
                let mut data_point = DataPoint::labeled_value(mountpoint.clone(), used_percent);
                data_point.description = format!("{} | {} / {} used", fs_type, used_h, size_h);
                data_point.command_params.push(mountpoint);
                result.multivalue.push(data_point);

            }

            Ok(result)
        }
        else {
            self.error_unsupported()
        }
    }
}