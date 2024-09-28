
extern crate qmetaobject;
use std::time::SystemTime;

use qmetaobject::*;

use crate::{
    error::LkError,
    module::monitoring::DataPoint,
    pro_service,
};


#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct ChartManagerModel {
    base: qt_base_class!(trait QObject),

    dataReceived: qt_signal!(invocation_id: u64, chart_data: QString),

    pro_service: Option<pro_service::ProService>,
}

impl ChartManagerModel {
    pub fn new(pro_service: Option<pro_service::ProService>) -> ChartManagerModel {
        ChartManagerModel {
            pro_service: pro_service,
            ..Default::default()
        }
    }

    pub fn stop(&mut self) {
        if let Some(pro_service) = self.pro_service.as_mut() {
            // TODO: notify UI?
            if let Err(error) = pro_service.stop() {
                ::log::error!("Error stopping Pro Service: {:?}", error);
            }
        }
    }

    pub fn insert_data_point(&mut self, host_id: &str, monitor_id: &str, data_point: DataPoint) -> Result<u64, LkError> {
        if let Some(pro_service) = self.pro_service.as_mut() {
            let current_unix_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;

            let mut metrics = vec![pro_service::Metric {
                label: data_point.label.clone(),
                value: data_point.value_int,
                time: current_unix_ms
            }];

            for child in data_point.multivalue.iter() {
                metrics.push(pro_service::Metric {
                    label: child.label.clone(),
                    value: child.value_int,
                    time: current_unix_ms
                });
            }

            let invocation_id = pro_service.send_request(pro_service::RequestType::MetricsInsert {
                    host_id: host_id.to_string(),
                    monitor_id: monitor_id.to_string(),
                    metrics: metrics,
            });

            Ok(invocation_id)
        }
        else {
            Err(LkError::other("Pro Service is not available."))
        }
    }

    pub fn refresh_chart_data(&mut self, host_id: &String) -> Result<u64, LkError> {
        if let Some(pro_service) = self.pro_service.as_mut() {
            let current_unix_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

            let invocation_id = pro_service.send_request(pro_service::RequestType::MetricsQuery {
                    host_id: host_id.clone(),
                    start_time: current_unix_time.as_secs() as i64 - 60 * 60 * 24,
                    end_time: current_unix_time.as_secs() as i64,
            });

            Ok(invocation_id)
        }
        else {
            Err(LkError::other("Pro Service is not available."))
        }
    }

    pub fn process_response(&mut self, response: pro_service::ServiceResponse) {
        let chart_data = serde_json::to_string(&response.metrics).unwrap();
        self.dataReceived(response.request_id.into(), chart_data.into());
    }
}