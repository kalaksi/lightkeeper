
extern crate qmetaobject;
use std::time::SystemTime;

use qmetaobject::*;

use crate::{error::LkError, module::monitoring::DataPoint, pro_services};


#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct ChartManagerModel {
    base: qt_base_class!(trait QObject),
    /// Every request gets an invocation ID. Valid numbers begin from 1.
    invocation_id_counter: u32,
    /// Handles communication with Lightkeeper Pro service.
    pro_service: pro_services::ProService,
}

impl ChartManagerModel {
    pub fn new() -> ChartManagerModel {
        ChartManagerModel {
            invocation_id_counter: 1,
            pro_service: pro_services::ProService::new(),
            ..Default::default()
        }
    }

    pub fn configure(&mut self) {
        if let Err(error) = self.pro_service.start() {
            ::log::error!("Failed to start Lightkeeper Pro service: {}", error);
            ::log::error!("Pro features will not be available.");
            // TODO: signal to UI / send error message
        };
    }

    pub fn stop(&mut self) {
        if self.pro_service.is_available() {
            self.pro_service.stop();
        }
    }

    pub fn insert_data_point(&mut self, host_id: &String, data_point: DataPoint) -> Result<u32, LkError> {
        let invocation_id = self.invocation_id_counter;
        self.invocation_id_counter += 1;

        let current_unix_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;
        let mut metrics = vec![pro_services::Metric {
            label: data_point.label.clone(),
            value: data_point.value_int,
            time: current_unix_ms
        }];

        for child in data_point.multivalue.iter() {
            metrics.push(pro_services::Metric {
                label: child.label.clone(),
                value: child.value_int,
                time: current_unix_ms
            });
        }

        self.pro_service.process_request(pro_services::ServiceRequest {
            request_id: invocation_id,
            time: current_unix_ms as u32,
            request_type: pro_services::RequestType::MetricsInsert {
                host_id: host_id.clone(),
                monitor_id: monitor_id.clone(),
                metrics: metrics,
            },
        })?;

        self.state_update_sender.as_ref().unwrap().send(StateUpdateMessage {
            host_name: host_id.to_owned(),
            module_spec: monitor.get_module_spec(),
            data_point: Some(data_point),
            invocation_id: current_invocation_id,
            ..Default::default()
        })?;

        Ok(invocation_id)
    }

    pub fn refresh_chart_data(&mut self, host_id: &String) {
        if self.pro_service.is_available() {
            let current_unix_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
            self.invocation_id_counter += 1;

            let response = self.pro_service.process_request(pro_services::ServiceRequest {
                request_id: self.invocation_id_counter as u32,
                time: current_unix_time.as_millis() as u32,
                request_type: pro_services::RequestType::MetricsQuery {
                    host_id: host_id.clone(),
                    start_time: current_unix_time.as_secs() as i64 - 60 * 60 * 24,
                    end_time: current_unix_time.as_secs() as i64,
                }
            });

            if let Err(error) = response {
                ::log::error!("Failed to get chart data: {}", error);
                // TODO: send error to UI?
            }
        }

        self.state_update_sender.as_ref().unwrap().send(StateUpdateMessage {
            host_name: host_id.to_owned(),
            module_spec: monitor.get_module_spec(),
            data_point: Some(DataPoint::pending()),
            invocation_id: current_invocation_id,
            ..Default::default()
        })?;
        Ok(())
    }


}