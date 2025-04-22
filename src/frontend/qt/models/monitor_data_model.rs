/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;

#[derive(Default, Clone)]
pub struct MonitorDataModel {
    pub data: QVariantMap,
}

impl MonitorDataModel {
    pub fn new(host_display_data: &frontend::HostDisplayData) -> Self {
        let mut model = MonitorDataModel { 
            ..Default::default()
        };

        for (monitor_id, monitor_data) in &host_display_data.host_state.monitor_data {
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