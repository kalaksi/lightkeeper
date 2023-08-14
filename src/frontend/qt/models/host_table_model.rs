use std::collections::HashMap;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::HostDataModel;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
pub struct HostTableModel {
    base: qt_base_class!(trait QAbstractTableModel),
    display_data: qt_property!(QVariant; WRITE set_display_data),
    data_changed_for_host: qt_method!(fn(&self, host_id: QString)),

    // Remove host without application restart.
    remove_host: qt_method!(fn(&mut self, host_id: QString)),

    // toggle_row is preferred for setting selected row.
    selected_row: qt_property!(i32; NOTIFY selected_row_changed),
    selected_row_changed: qt_signal!(),
    selection_activated: qt_signal!(),
    selection_deactivated: qt_signal!(),
    toggle_row: qt_method!(fn(&mut self, row: i32)),
    get_selected_host_id: qt_method!(fn(&self) -> QString),

    headers: Vec<QString>,
    host_row_map: HashMap<String, usize>,
    i_display_data: frontend::DisplayData,
    // Currently stores the same data as HostDataManagerModel but that might change.
    /// Holds preprocessed data more fitting for table rows.
    row_data: Vec<HostDataModel>,
    disabled_hosts: Vec<String>,
}

impl HostTableModel {
    fn set_display_data(&mut self, display_data: QVariant) {
        self.begin_reset_model();

        self.selected_row = -1;
        self.i_display_data = frontend::DisplayData::from_qvariant(display_data).unwrap();
        self.headers = self.i_display_data.table_headers.iter().map(|header| QString::from(header.clone())).collect::<Vec<QString>>();

        let mut host_ids_ordered = self.i_display_data.hosts.keys().collect::<Vec<&String>>();
        host_ids_ordered.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        // Ignore disabled hosts.
        host_ids_ordered.retain(|host_id| !self.disabled_hosts.contains(*host_id));

        self.host_row_map.clear();
        self.row_data.clear();
        for host_id in host_ids_ordered {
            let host_data = self.i_display_data.hosts.get(host_id).unwrap();
            self.host_row_map.insert(host_id.clone(), self.row_data.len());
            self.row_data.push(HostDataModel::from(&host_data));
        }

        self.end_reset_model();
    }

    // A slot for informing about change in table data.
    fn data_changed_for_host(&mut self, host_id: QString) {
        let host_index = self.host_row_map.get(&host_id.to_string()).unwrap();

        let top_left = self.index(*host_index as i32, 0);
        let bottom_right = self.index(*host_index as i32, self.column_count() as i32 - 1);

        // The standard Qt signal.
        self.data_changed(top_left, bottom_right);
    }

    fn toggle_row(&mut self, row: i32) {
        if self.selected_row == row {
            self.selected_row = -1;
            self.selection_deactivated();
        }
        else {
            if self.selected_row == -1 {
                self.selection_activated();
            }
            self.selected_row = row;
        }
        self.selected_row_changed();
    }

    fn get_selected_host_id(&self) -> QString {
        match self.row_data.get(self.selected_row as usize) {
            Some(host) => host.name.clone(),
            None => QString::from(""),
        }
    }

    fn remove_host(&mut self, host_id: QString) {
        self.disabled_hosts.push(host_id.to_string());
        let host_index = self.host_row_map.remove(&host_id.to_string()).unwrap() as i32;

        for row in self.host_row_map.values_mut() {
            if *row > host_index as usize {
                *row -= 1;
            }
        }

        self.toggle_row(self.selected_row);
        self.begin_remove_rows(host_index, host_index);
        self.row_data.retain(|host| host.name != host_id);
        self.end_remove_rows();
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
            5 => row.name.to_qvariant(),
            _ => panic!(),
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}