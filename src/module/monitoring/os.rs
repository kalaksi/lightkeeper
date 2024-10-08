
use std::collections::HashMap;

use lightkeeper_module::monitoring_module;
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="os",
    version="0.0.1",
    description="Provides basic information about the operating system."
)]
pub struct Os;

impl Module for Os {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Os { }
    }
}

impl MonitoringModule for Os {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Operating system"),
            category: String::from("host"),
            use_without_summary: true,
            ..Default::default()
        }
    }

    fn process_response(&self, host: Host, _response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        Ok(DataPoint::new(format!("{} ({} {})", host.platform.os, host.platform.os_flavor, host.platform.os_version)))
    }
}