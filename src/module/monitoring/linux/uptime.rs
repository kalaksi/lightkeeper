
use std::collections::HashMap;
use chrono::{ NaiveDateTime, Utc };
use crate::module::connection::ResponseMessage;
use crate::{
    Host,
    frontend,
};
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module("uptime", "0.0.1")]
pub struct Uptime;

impl Module for Uptime {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Uptime { }
    }
}

impl MonitoringModule for Uptime {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Uptime"),
            category: String::from("host"),
            unit: String::from("days"),
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self) -> String {
        String::from("uptime -s")
    }

    fn process_response(&self, _host: Host, response: ResponseMessage) -> Result<DataPoint, String> {
        let boot_datetime = NaiveDateTime::parse_from_str(&response.message, "%Y-%m-%d %H:%M:%S")
                                          .map_err(|e| e.to_string())?;

        let uptime = Utc::now().naive_utc() - boot_datetime;
        Ok(DataPoint::new(uptime.num_days().to_string()))
    }
}