
use std::collections::HashMap;
use std::net::{TcpStream, SocketAddr};
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
        timeout => "How many seconds to wait for connection. Default: 10.",
    }
)]
pub struct TcpConnect {
    port: u16,
    timeout: u8,
}

impl Module for TcpConnect {
    fn new(_settings: &HashMap<String, String>) -> Self {
        TcpConnect {
            port: _settings.get("port").and_then(|value| Some(value.parse().unwrap())).unwrap_or(22),
            timeout: _settings.get("timeout").and_then(|value| Some(value.parse().unwrap())).unwrap_or(10),
        }
    }
}

impl MonitoringModule for TcpConnect {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::StatusUpDown,
            display_text: format!("TCP port {}", self.port),
            category: String::from("network"),
            ..Default::default()
        }
    }

    fn process_response(&self, host: Host, _response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        let socket_addr: SocketAddr = format!("{}:{}", host.ip_address, self.port).parse().unwrap();
        let result = TcpStream::connect_timeout(&socket_addr, std::time::Duration::from_secs(self.timeout as u64));

        if let Err(_) = result {
            Ok(DataPoint::value_with_level(String::from("down"), Criticality::Critical))
        }
        else {
            Ok(DataPoint::value_with_level(String::from("up"), Criticality::Normal))
        }
    }
}