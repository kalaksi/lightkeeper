/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;

/// HostData is the corresponding Qt struct for frontend::HostDisplayData.
/// Contains host and state information used by the host table.
#[derive(QGadget, Default, Clone)]
pub struct HostDataModel {
    pub status: qt_property!(QString),
    pub name: qt_property!(QString),
    pub fqdn: qt_property!(QString),
    pub ip_address: qt_property!(QString),
}

impl HostDataModel {
    pub fn from(host_display_data: &frontend::HostDisplayData) -> Self {
        HostDataModel {
            status: host_display_data.host_state.status.clone().to_string().into(),
            name: host_display_data.host_state.host.name.clone().into(),
            fqdn: host_display_data.host_state.host.fqdn.clone().into(),
            ip_address: host_display_data.host_state.host.ip_address.to_string().into(),
        }
    }
}
