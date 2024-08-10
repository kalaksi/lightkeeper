use x509_parser::prelude::Pem;

use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{ Host, enums::Criticality };
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="network-cert-monitor",
    version="0.0.1",
    description="Monitor TLS (HTTPS) certificate validity.",
    settings={
        threshold_warning => "Warning if certificate age is less than this many days. Default: 21",
        threshold_error => "Error if certificate age is less than this many days. Default: 14"
    }
)]
pub struct CertMonitor {
    threshold_warning: u16,
    threshold_error: u16,
}

impl Module for CertMonitor {
    fn new(settings: &HashMap<String, String>) -> Self {
        CertMonitor {
            // port: settings.get("port").map(|value| value.parse().unwrap()).unwrap_or(22),
            // timeout: settings.get("timeout").map(|value| value.parse().unwrap()).unwrap_or(10),
            threshold_warning: settings.get("threshold_warning").map(|value| value.parse().unwrap()).unwrap_or(21),
            threshold_error: settings.get("threshold_error").map(|value| value.parse().unwrap()).unwrap_or(14),
        }
    }
}

impl MonitoringModule for CertMonitor {
    fn process_response(&self, _host: Host, response: ResponseMessage, _result: DataPoint) -> Result<DataPoint, String> {
        if response.is_error() {
            return Ok(DataPoint::value_with_level(response.message, Criticality::Error));
        }

        // Fetch only the peer certificate (first one) for inspection.
        if let Some(pem_result) = Pem::iter_from_buffer(response.message.as_bytes()).into_iter().next() {
            match pem_result {
                Ok(pem) => {
                    if let Ok(x509_cert) = pem.parse_x509() {
                        let days_left = x509_cert.validity().time_to_expiration().unwrap_or_default().whole_days();

                        let error_level = if days_left <= self.threshold_error as i64 {
                            Criticality::Error
                        }
                        else if days_left <= self.threshold_warning as i64 {
                            Criticality::Warning
                        }
                        else {
                            Criticality::Info
                        };

                        Ok(DataPoint::value_with_level(format!("{}", days_left), error_level))
                    }
                    else {
                        Ok(DataPoint::value_with_level("Failed to parse PEM.".to_string(), Criticality::Error))
                    }
                },
                Err(error) => {
                    let error = format!("Failed to parse PEM: {}", error);
                    Ok(DataPoint::value_with_level(error, Criticality::Error))
                }
            }
        }
        else {
            Ok(DataPoint::value_with_level("No certificate received from peer.".to_string(), Criticality::Error))
        }
    }
}