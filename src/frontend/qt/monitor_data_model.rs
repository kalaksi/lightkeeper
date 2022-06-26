extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;

#[derive(Default, Clone)]
pub struct MonitorDataModel {
    pub data: QVariantList,
}

impl MonitorDataModel {
    pub fn new(host_display_data: &frontend::HostDisplayData) -> Self {
        let mut model = MonitorDataModel { 
            data: QVariantList::default(),
        };

        for (_, data) in &host_display_data.monitoring_data {
            model.data.push(serde_json::to_string(&data).unwrap().to_qvariant());
        }

        model
    }
}


impl QMetaType for MonitorDataModel {

}