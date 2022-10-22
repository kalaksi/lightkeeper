use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};
use crate::module::{
    Module,
    Metadata,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::DataPoint,
};

#[derive(Clone)]
pub struct Filesystem {
    ignored_filesystems: Vec<String>,
}

impl Module for Filesystem {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("filesystem", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Filesystem {
            ignored_filesystems: vec![
                String::from("/run"),
            ]
        }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Filesystem {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::ProgressBar,
            display_text: String::from("Filesystem usage"),
            category: String::from("host"),
            unit: String::from("%"),
            use_multivalue: true,
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self) -> String {
        String::from("df -P")
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();

        let mut lines = response.message.split('\n');
        // First line contains headers
        lines.nth(1).unwrap();
        for line in lines {
            let mut parts = line.split_whitespace();
            let _source = parts.next().unwrap().to_string();
            let _size_m = parts.next().unwrap().to_string();
            let _used_m = parts.next().unwrap().to_string();
            let _available_m = parts.next().unwrap().to_string();
            let mut used_percent = parts.next().unwrap().to_string();
            let mountpoint = parts.next().unwrap().to_string();

            if self.ignored_filesystems.iter().any(|item| mountpoint.starts_with(item)) {
                continue;
            }

            // Remove percent symbol from the end.
            used_percent.pop();
            let data_point = DataPoint::labeled_value(mountpoint, used_percent);
            result.multivalue.push(data_point);

        }

        Ok(result)
    }
}