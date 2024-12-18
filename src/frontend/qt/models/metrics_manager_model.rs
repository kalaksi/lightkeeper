
extern crate qmetaobject;
use std::time::SystemTime;

use qmetaobject::*;

use crate::{
    metrics::Metric,
    module::monitoring::DataPoint,
    metrics,
};


#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct MetricsManagerModel {
    base: qt_base_class!(trait QObject),

    //
    // Slots
    //
    refreshCharts: qt_method!(fn(&self, host_id: QString, monitor_id: QString) -> u64),


    //
    // Signals
    //
    dataReceived: qt_signal!(invocation_id: u64, chart_data: QString),

    //
    // Private properties
    //
    metrics_manager: Option<metrics::MetricsManager>,
}

#[allow(non_snake_case)]
impl MetricsManagerModel {
    pub fn new(metrics_manager: Option<metrics::MetricsManager>) -> MetricsManagerModel {
        MetricsManagerModel {
            metrics_manager,
            ..Default::default()
        }
    }

    pub fn stop(&mut self) {
        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            // TODO: notify UI?
            if let Err(error) = metrics_manager.stop() {
                ::log::error!("Error stopping metrics server: {:?}", error);
            }
        }
    }

    pub fn insert_data_point(&mut self, host_id: &str, monitor_id: &str, data_point: DataPoint) {
        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            let current_unix_ms = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;

            let mut metrics = vec![Metric {
                label: data_point.label.clone(),
                value: data_point.value_int,
                time: current_unix_ms
            }];

            for child in data_point.multivalue.iter() {
                metrics.push(Metric {
                    label: child.label.clone(),
                    value: child.value_int,
                    time: current_unix_ms
                });
            }

            let invocation_result = metrics_manager.insert_metrics(host_id, monitor_id, &metrics);
            if let Err(error) = invocation_result {
                ::log::error!("Error inserting data point: {:?}", error);
            }
        }
    }

    pub fn process_update(&mut self, response: metrics::tmserver::TMSResponse) {
        let chart_data = serde_json::to_string(&response.metrics).unwrap();
        self.dataReceived(response.request_id.into(), chart_data.into());
    }

    fn refreshCharts(&mut self, host_id: QString, monitor_id: QString) -> u64 {
        if let Some(metrics_manager) = self.metrics_manager.as_mut() {
            let current_unix_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

            let invocation_result = metrics_manager.get_metrics(
                &host_id.to_string(),
                &monitor_id.to_string(),
                // 1 day back.
                current_unix_time.as_secs() as i64 - 60 * 60 * 24,
                current_unix_time.as_secs() as i64,
            );

            match invocation_result {
                Ok(invocation_id) => invocation_id,
                Err(error) => {
                    ::log::error!("Error refreshing charts: {:?}", error);
                    0
                }
            }
        }
        else {
            0
        }
    }
}