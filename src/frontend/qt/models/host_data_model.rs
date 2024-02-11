extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::monitor_data_model::MonitorDataModel;

/// HostData is the corresponding Qt struct for frontend::HostDisplayData.
/// Contains host and state information.
#[derive(QGadget, Default, Clone)]
pub struct HostDataModel {
    pub status: qt_property!(QString),
    pub name: qt_property!(QString),
    pub fqdn: qt_property!(QString),
    pub ip_address: qt_property!(QString),
    pub monitor_data: qt_property!(MonitorDataModel),
}

impl HostDataModel {
    pub fn from(host_display_data: &frontend::HostDisplayData) -> Self {
        HostDataModel {
            status: host_display_data.host_state.status.clone().to_string().into(),
            name: host_display_data.host_state.host.name.clone().into(),
            fqdn: host_display_data.host_state.host.fqdn.clone().into(),
            ip_address: host_display_data.host_state.host.ip_address.to_string().into(),
            monitor_data: MonitorDataModel::new(&host_display_data),
        }
    }
}