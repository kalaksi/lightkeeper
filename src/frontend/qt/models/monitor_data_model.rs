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
}


impl QMetaType for MonitorDataModel {

}