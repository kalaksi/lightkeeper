
use std::collections::HashMap;
use chrono::{ NaiveDateTime, Utc };
use crate::module::connection::ResponseMessage;
use crate::{
    utils::strip_newline,
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
pub struct Uptime;

impl Module for Uptime {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("uptime", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Uptime { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl MonitoringModule for Uptime {
    fn clone_module(&self) -> Monitor {
        Box::new(self.clone())
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Uptime"),
            category: String::from("host"),
            unit: String::from("d"),
            ..Default::default()
        }
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_connector_message(&self) -> String {
        String::from("uptime -s")
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _connector_is_connected: bool) -> Result<DataPoint, String> {
        let boot_datetime = NaiveDateTime::parse_from_str(&strip_newline(&response.message), "%Y-%m-%d %H:%M:%S")
                                          .map_err(|e| e.to_string())?;

        let uptime = Utc::now().naive_utc() - boot_datetime;
        Ok(DataPoint::new(uptime.num_days().to_string()))
    }
}