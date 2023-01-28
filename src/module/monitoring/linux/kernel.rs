
use std::collections::HashMap;

use lightkeeper_module::monitoring_module;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("kernel", "0.0.1")]
pub struct Kernel;

impl Module for Kernel {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Kernel { }
    }
}

impl MonitoringModule for Kernel {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Kernel version"),
            category: String::from("host"),
            ignore_from_summary: true,
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self, host: Host) -> String {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            String::from("uname -r -m")
        }
        else {
            String::new()
        }
    }

    fn process_response(&self, host: Host, response: ResponseMessage) -> Result<DataPoint, String> {
        if host.platform.os == platform_info::OperatingSystem::Linux {
            Ok(DataPoint::new(response.message.replace(" ", " (") + ")"))
        }
        else {
            self.error_unsupported()
        }
    }
}