use x509_parser::prelude::Pem;

use std::collections::HashMap;
use crate::module::connection::ResponseMessage;
use crate::{ Host, enums::Criticality };
use crate::error::LkError;
use lightkeeper_module::monitoring_module;
use crate::module::*;
use crate::module::monitoring::*;

#[monitoring_module(
    name="_cert-monitor",
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
    addresses: Vec<String>,
}

impl Module for CertMonitor {
    fn new(settings: &HashMap<String, String>) -> Self {
        CertMonitor {
            threshold_warning: settings.get("threshold_warning").map(|value| value.parse().unwrap_or_default()).unwrap_or(21),
            threshold_error: settings.get("threshold_error").map(|value| value.parse().unwrap_or_default()).unwrap_or(14),
            addresses: settings.get("addresses").map(|value| value.split(',').map(|s| s.to_string()).collect()).unwrap_or_default(),
        }
    }
}

impl MonitoringModule for CertMonitor {
    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::connector("tcp", "0.0.1"))
    }

    fn get_connector_messages(&self, _host: Host, _result: DataPoint) -> Result<Vec<String>, LkError> {
        Ok(self.addresses.clone())
    }

    fn process_responses(&self, _host: Host, responses: Vec<ResponseMessage>, _result: DataPoint) -> Result<DataPoint, String> {
        let mut result = DataPoint::empty();

        let addresses_and_responses = self.addresses.iter().zip(responses.iter());

        for (address, response) in addresses_and_responses {
            let child = if response.is_error() {
                DataPoint::value_with_level(response.message.clone(), Criticality::Error)
            }
            else {
                // Fetch only the peer certificate (first one) for inspection.
                if let Some(pem_result) = Pem::iter_from_buffer(response.message.as_bytes()).into_iter().next() {
                    match pem_result {
                        Ok(pem) => {
                            if let Ok(x509_cert) = pem.parse_x509() {
                                let days_left = x509_cert.validity().time_to_expiration().unwrap_or_default().whole_days();
                                let common_name = x509_cert.subject.to_string();
                                let issuer = x509_cert.issuer.to_string();
                                let description = format!("{} | Issuer: {}", common_name, issuer);

                                let error_level = if days_left <= self.threshold_error as i64 {
                                    Criticality::Error
                                }
                                else if days_left <= self.threshold_warning as i64 {
                                    Criticality::Warning
                                }
                                else {
                                    Criticality::Info
                                };

                                DataPoint::labeled_value_with_level(address.clone(), format!("{}", days_left), error_level)
                                          .with_description(description)
                            }
                            else {
                                DataPoint::labeled_value_with_level(address.clone(), "Failed to parse PEM.".to_string(), Criticality::Error)
                            }
                        },
                        Err(error) => {
                            let error = format!("Failed to parse PEM: {}", error);
                            DataPoint::labeled_value_with_level(address.clone(), error, Criticality::Error)
                        }
                    }
                }
                else {
                    DataPoint::labeled_value_with_level(address.clone(), "No certificate received.".to_string(), Criticality::Error)
                }
            };

            result.multivalue.push(child);
        }

        Ok(result)
    }
}