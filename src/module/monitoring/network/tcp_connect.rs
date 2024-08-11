
use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{ Host, enums::Criticality, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="tcp-connect",
    version="0.0.1",
    description="Tests connecting to a specified port via TCP.",
    settings={
        port => "Port to connect to. Default: 22.",
    }
)]
pub struct TcpConnect {
    port: u16,
}

impl Module for TcpConnect {
    fn new(settings: &HashMap<String, String>) -> Self {
        TcpConnect {
            port: settings.get("port").map(|value| value.parse().unwrap()).unwrap_or(22),
        }
    }
}

impl MonitoringModule for TcpConnect {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("tcp", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::CriticalityLevel,
            display_text: format!("TCP port {}", self.port),
            category: String::from("network"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, host: Host, _parent_result: DataPoint) -> Result<String, crate::error::LkError> {
        Ok(format!("{}:{}", host.ip_address, self.port))
    }

    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.is_error() {
            return Ok(DataPoint::value_with_level(response.message, Criticality::Error));
        }
        else {
            Ok(DataPoint::new(String::from("open")))
        }
    }
}