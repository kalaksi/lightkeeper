use std::{
    time::Duration,
};

use crate::module::{
    module::Module,
    Metadata,
    connection::ConnectionModule,
    monitoring::{MonitoringModule, MonitoringData},
};

pub struct Uptime {
}

impl Module for Uptime {
    fn get_metadata() -> Metadata {
        Metadata {
            name: String::from("uptime"),
            version: String::from("0.0.1"),
            interface_version: String::from("0.0.1"),
            display_name: String::from("Uptime"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new() -> Self {
        Uptime { }
    }

    fn unload(&self) {

    }
}

impl MonitoringModule for Uptime {
    fn refresh(&self, connection: &mut Box<dyn ConnectionModule>) -> Result<MonitoringData, String> {
        let output = match connection.send_message("uptime") {
            Ok(output) => output,
            Err(error) => return Err(error)
        };

        Ok(MonitoringData {
            value: output,
            unit: String::from("d"),
            retention: Duration::from_secs(1),
        })
    }
}