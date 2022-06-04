
use std::collections::HashMap;
use chrono::{ NaiveDateTime, Utc };
use crate::{
    utils::strip_newline,
    Host,
};
use crate::module::{
    Module,
    Metadata,
    connection::ConnectionModule,
    monitoring::{MonitoringModule, MonitoringData},
    ModuleSpecification,
};

pub struct Uptime {
}

impl Module for Uptime {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new(String::from("uptime"), String::from("0.0.1")),
            display_name: String::from("Uptime"),
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
    fn get_connector_spec(&self) -> ModuleSpecification {
        ModuleSpecification::new(String::from("ssh"), String::from("0.0.1"))
    }

    fn refresh(&mut self, _host: &Host, connection: &mut Box<dyn ConnectionModule>) -> Result<MonitoringData, String> {
        let output = strip_newline(&connection.send_message("uptime -s")?);
        let boot_datetime = NaiveDateTime::parse_from_str(&output, "%Y-%m-%d %H:%M:%S").map_err(|e| e.to_string())?;
        let uptime = Utc::now().naive_utc() - boot_datetime;

        Ok(MonitoringData::new(uptime.num_days().to_string(), String::from("d")))
    }
}