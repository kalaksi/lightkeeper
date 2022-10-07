extern crate qmetaobject;
use qmetaobject::*;
use std::collections::HashMap;

use crate::frontend;
use crate::module::monitoring::MonitoringData;

#[derive(Default, Clone)]
pub struct MonitorDataModel {
    pub data: QVariantMap,
}

impl MonitorDataModel {
    pub fn new(host_display_data: &frontend::HostDisplayData) -> Self {
        let mut model = MonitorDataModel { 
            ..Default::default()
        };

        for (monitor_id, monitor_data) in &host_display_data.monitoring_data {
            model.data.insert(
                QString::from(monitor_id.clone()),
                serde_json::to_string(&monitor_data).unwrap().to_qvariant()
            );
        }

        model
    }

    pub fn deserialize(&self) -> HashMap<String, MonitoringData> {
        let mut result = HashMap::new();

        for (monitor_id, monitor_data_json) in self.data.into_iter() {
            let text = monitor_data_json.to_qbytearray().to_string();

            if !text.is_empty() {
                let deserialized = serde_json::from_str(&text).unwrap();
                result.insert(monitor_id.to_string(), deserialized);
            }
        }

        result
    }
}


impl QMetaType for MonitorDataModel {

}