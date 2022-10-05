use std::collections::HashMap;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::HostDataModel;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
pub struct HostTableModel {
    base: qt_base_class!(trait QAbstractTableModel),
    headers: Vec<QString>,
    // Currently stores the same data as HostDataManagerModel but that might change.
    row_data: Vec<HostDataModel>,
    host_row_map: HashMap<String, usize>,

    update: qt_method!(fn(&mut self, new_host_data: HostDataModel)),
    data_changed_for_host: qt_method!(fn(&self, host_id: QString)),

    selected_row: qt_property!(i32; NOTIFY selected_row_changed),
    selected_row_changed: qt_signal!(),
    get_selected_host_id: qt_method!(fn(&self) -> QString),
}

impl HostTableModel {
    pub fn new(display_data: &frontend::DisplayData) -> Self {
        let mut model = HostTableModel {
            headers: Vec::new(),
            row_data: Vec::new(),
            host_row_map: HashMap::new(),
            selected_row: -1,
            ..Default::default()
        };

        for header in &display_data.table_headers {
            model.headers.push(header.clone().into());
        }

        for (host_id, host_data) in display_data.hosts.iter() {
            model.host_row_map.insert(host_id.clone(), model.row_data.len());
            model.row_data.push(HostDataModel::from(&host_data));
        }

        model
    }

    // TODO: currently unused, clean up maybe
    pub fn update(&mut self, new_host_data: HostDataModel) {
        let host_index = self.host_row_map.get(&new_host_data.name.to_string()).unwrap();
        let old_value = std::mem::replace(self.row_data.get_mut(*host_index).unwrap(), new_host_data);
        self.data_changed_for_host(old_value.name);
    }

    pub fn data_changed_for_host(&mut self, host_id: QString) {
        let host_index = self.host_row_map.get(&host_id.to_string()).unwrap();

        let top_left = self.index(*host_index as i32, 0);
        let bottom_right = self.index(*host_index as i32, self.column_count() as i32 - 1);

        // The standard Qt signal.
        self.data_changed(top_left, bottom_right);
    }

    pub fn get_selected_host_id(&self) -> QString {
        match self.row_data.get(self.selected_row as usize) {
            Some(host) => host.name.clone(),
            None => QString::from(String::new()),
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
            5 => row.name.to_qvariant(),
            _ => panic!(),
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}