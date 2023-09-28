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
    dataChangedForHost: qt_method!(fn(&self, host_id: QString)),

    // toggleRow is preferred for setting selected row.
    selectedRow: qt_property!(i32; NOTIFY selectedRowChanged),
    selectedRowChanged: qt_signal!(),
    selectionActivated: qt_signal!(),
    selectionDeactivated: qt_signal!(),
    toggleRow: qt_method!(fn(&mut self, row: i32)),
    getSelectedHostId: qt_method!(fn(&self) -> QString),

    host_row_map: HashMap<String, usize>,
    i_display_data: frontend::DisplayData,
    // Currently stores the same data as HostDataManagerModel but that might change.
    /// Holds preprocessed data more fitting for table rows.
    row_data: Vec<HostDataModel>,
}

#[allow(non_snake_case)]
impl HostTableModel {
    fn set_display_data(&mut self, display_data: QVariant) {
        self.begin_reset_model();

        self.i_display_data = frontend::DisplayData::from_qvariant(display_data).unwrap();

        let mut host_ids_ordered = self.i_display_data.hosts.keys().collect::<Vec<&String>>();
        host_ids_ordered.sort_by_key(|key| key.to_lowercase());

        self.host_row_map.clear();
        self.row_data.clear();
        for host_id in host_ids_ordered {
            let host_data = self.i_display_data.hosts.get(host_id).unwrap();
            self.host_row_map.insert(host_id.clone(), self.row_data.len());
            self.row_data.push(HostDataModel::from(host_data));
        }

        self.end_reset_model();

        // Remember currently selected host. If missing, then go back to -1.
        let selected_host_id = self.getSelectedHostId();
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
        match self.row_data.get(self.selectedRow as usize) {
            Some(host) => host.name.clone(),
            None => QString::from(""),
        }
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
            0 => row.status.to_qvariant(),
            1 => row.name.to_qvariant(),
            2 => row.fqdn.to_qvariant(),
            3 => row.ip_address.to_qvariant(),
            // Return host id to use with get_monitor_data().
            4 => row.name.to_qvariant(),
            _ => panic!(),
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}