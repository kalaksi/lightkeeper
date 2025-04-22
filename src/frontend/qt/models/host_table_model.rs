/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::HostDataModel;


#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct HostTableModel {
    base: qt_base_class!(trait QAbstractTableModel),
    displayData: qt_property!(QVariant; WRITE set_display_data),
    // toggleRow is preferred for setting selected row.
    selectedRow: qt_property!(i32; NOTIFY selectedRowChanged),
    rowCount: qt_property!(i32; READ row_count),

    selectedRowChanged: qt_signal!(),
    selectionActivated: qt_signal!(),
    selectionDeactivated: qt_signal!(),

    dataChangedForHost: qt_method!(fn(&self, host_id: QString)),
    toggleRow: qt_method!(fn(&mut self, row: i32)),
    getSelectedHostId: qt_method!(fn(&self) -> QString),
    filter: qt_method!(fn(&self, filter: QString)),

    host_row_map: HashMap<String, usize>,
    i_display_data: frontend::DisplayData,
    // Currently stores the same data as HostDataManagerModel but that might change.
    /// Holds preprocessed data more fitting for table rows.
    row_data: Vec<HostDataModel>,
    search_filter: String,
}

#[allow(non_snake_case)]
impl HostTableModel {
    fn set_display_data(&mut self, display_data: QVariant) {
        // Remember currently selected host.
        let selected_host_id = self.getSelectedHostId();

        self.begin_reset_model();
        self.i_display_data = frontend::DisplayData::from_qvariant(display_data).unwrap();
        self.update_row_data();
        self.end_reset_model();

        self.selectedRow = self.host_row_map.get(&selected_host_id.to_string())
                                            .map(|row| *row as i32).unwrap_or(-1);

        if self.selectedRow >= 0 {
            self.selectionActivated();
        }
    }

    fn update_row_data(&mut self) {
        let mut filtered_hosts = self.i_display_data.hosts.iter()
            // Host IDs starting with underscore are reserved for internal use.
            .filter(|(host_id, _)| !host_id.starts_with("_"))
            .filter(|(host_id, host_display_data)| {
                self.search_filter.is_empty() ||
                host_id.to_lowercase().contains(&self.search_filter) ||
                host_display_data.host_state.host.fqdn.to_lowercase().contains(&self.search_filter) ||
                host_display_data.host_state.host.ip_address.to_string().to_lowercase().contains(&self.search_filter)
        }).collect::<Vec<_>>();

        filtered_hosts.sort_by_key(|(key, _)| key.to_lowercase());

        // Remember currently selected host.
        let selected_host_id = self.getSelectedHostId();

        self.host_row_map.clear();
        self.row_data.clear();

        for (host_id, host_data) in filtered_hosts {
            self.host_row_map.insert(host_id.clone(), self.row_data.len());
            self.row_data.push(HostDataModel::from(host_data));
        }

        // Restore host selection.
        self.selectedRow = match self.host_row_map.get(&selected_host_id.to_string()) {
            Some(row) => *row as i32,
            None => -1,
        };

        if self.selectedRow >= 0 {
            self.selectionActivated();
        }
    }

    // A slot for informing about change in table data.
    fn dataChangedForHost(&mut self, host_id: QString) {
        let host_id = host_id.to_string();

        // Host might already be removed, so ignore. 
        if let Some(host_index) = self.host_row_map.get(&host_id) {
            let top_left = self.index(*host_index as i32, 0);
            let bottom_right = self.index(*host_index as i32, self.column_count() - 1);

            // The standard Qt signal.
            self.data_changed(top_left, bottom_right);
        }
    }

    fn toggleRow(&mut self, row: i32) {
        if self.selectedRow == row {
            self.selectedRow = -1;
            self.selectionDeactivated();
        }
        else {
            if self.selectedRow == -1 {
                self.selectionActivated();
            }
            self.selectedRow = row;
        }
        self.selectedRowChanged();
    }

    fn getSelectedHostId(&self) -> QString {
        if self.selectedRow >= 0 {
            match self.row_data.get(self.selectedRow as usize) {
                Some(host) => host.name.clone(),
                None => QString::from(""),
            }
        }
        else {
            QString::from("")
        }
    }

    fn filter(&mut self, filter: QString) {
        self.search_filter = filter.to_string().to_lowercase();
        self.begin_reset_model();
        self.update_row_data();
        self.end_reset_model();
    }

    fn row_count(&self) -> i32 {
        self.row_data.len() as i32
    }
}

impl QAbstractTableModel for HostTableModel {
    fn row_count(&self) -> i32 {
        self.row_data.len() as i32
    }

    fn column_count(&self) -> i32 {
        5
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role != USER_ROLE {
            return QString::from(format!("Unknown role: {}", role)).to_qvariant();
        }

        let row = self.row_data.get(index.row() as usize).unwrap();

        match index.column() {
            0 => row.status.to_lower().to_qvariant(),
            1 => row.name.to_qvariant(),
            2 => row.fqdn.to_qvariant(),
            3 => row.ip_address.to_qvariant(),
            // Return host id to use with different methods.
            4 => row.name.to_qvariant(),
            _ => panic!(),
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}