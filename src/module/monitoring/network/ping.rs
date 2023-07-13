
use std::collections::HashMap;
use oping;
use crate::module::connection::ResponseMessage;
use crate::{ Host, enums::Criticality, frontend };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    "ping",
    "0.0.1",
    "Measures latency to host with ICMP echo request.
    Settings: none"
)]
pub struct Ping;

impl Module for Ping {
    fn new(_settings: &HashMap<String, String>) -> Self {
        Ping { }
    }
}

impl MonitoringModule for Ping {
    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_style: frontend::DisplayStyle::Text,
            display_text: String::from("Ping"),
            category: String::from("network"),
            unit: String::from("ms"),
            ..Default::default()
        }
    }

    fn process_response(&self, host: Host, _responses: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        let mut ping = oping::Ping::new();

        ping.set_timeout(5.0)
            .map_err(|e| e.to_string())?;

        ping.add_host(host.ip_address.to_string().as_str())
            .map_err(|e| e.to_string())?;

        let mut responses = ping.send()
                                .map_err(|e| e.to_string())?;

        let response = responses.next().unwrap();

        if response.latency_ms < 0.0 {
            Ok(DataPoint::value_with_level(String::from("-"), Criticality::Critical))
        }
        else {
            Ok(DataPoint::value_with_level(response.latency_ms.to_string(), Criticality::Normal))
        }

    }
}