use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;

use super::host_data_model::HostDataModel;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
pub struct HostListModel {
    base: qt_base_class!(trait QAbstractTableModel),
    headers: Vec<QString>,
    hosts: HashMap<String, HostDataModel>,
    hosts_index: HashMap<usize, String>,

    receive_updates: qt_method!(fn(&self)),
    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,

    // TODO: separate data that is not strictly related to table data.
    // NOTE: Couldn't get custom types to work for return types,
    // so for now methods are used to get the data in JSON and parsed in QML.
    get_monitor_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_monitor_data_map: qt_method!(fn(&self, host_id: QString) -> QVariantMap),
    get_command_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_host_data: qt_method!(fn(&self, index: i32) -> QVariantMap),

    // For table row selection.
    selected_row: qt_property!(i32; NOTIFY selected_row_changed),
    selected_row_changed: qt_signal!(),
    get_selected_host: qt_method!(fn(&self) -> QString),
}

impl HostListModel {
    pub fn new(display_data: &frontend::DisplayData) -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let (sender, receiver) = mpsc::channel::<frontend::HostDisplayData>();
        let mut model = HostListModel {
            headers: Vec::new(),
            hosts: HashMap::new(),
            hosts_index: HashMap::new(),
            update_receiver: Some(receiver),
            update_receiver_thread: None,
            selected_row: -1,
            ..Default::default()
        };

        for header in &display_data.table_headers {
            model.headers.push(header.clone().into());
        }

        for (host_id, host_data) in display_data.hosts.iter() {
            model.hosts_index.insert(model.hosts.len(), host_id.clone()); model.hosts.insert(host_id.clone(), HostDataModel::from(&host_data));
        }

        (model, sender)
    }

    fn receive_updates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        if self.update_receiver_thread.is_none() {
            let self_ptr = QPointer::from(&*self);
            let set_data = qmetaobject::queued_callback(move |host_display_data: frontend::HostDisplayData| {
                self_ptr.as_pinned().map(|self_pinned| {
                    // HostDataModel cannot be passed between threads so parsing happens in set_data().
                    let host_data = HostDataModel::from(&host_display_data);

                    let _old_value = std::mem::replace(
                        self_pinned.borrow_mut().hosts.get_mut(&host_data.name.to_string()).unwrap(),
                        host_data,
                    );

                    // TODO:
                    // let index = self_pinned.borrow().hosts.data.iter().position(|&item| item.name == host_data.name).unwrap();
                    let top_left = self_pinned.borrow().index(0, 0);
                    let bottom_right = self_pinned.borrow().index(
                        self_pinned.borrow().hosts.len() as i32 - 1,
                        self_pinned.borrow().column_count() as i32 - 1
                    );
                    self_pinned.borrow_mut().data_changed(top_left, bottom_right);
                });
            });

            let receiver = self.update_receiver.take().unwrap();
            let thread = std::thread::spawn(move || {
                loop {
                    let received_data = receiver.recv().unwrap();
                    set_data(received_data);
                }
            });

            self.update_receiver_thread = Some(thread);
        }
    }

    // Returns list of MonitorData structs in JSON.
    fn get_monitor_data(&self, host_id: QString) -> QVariantList {
        if let Some(host) = self.hosts.get(&host_id.to_string()) {
            host.monitor_data.data.into_iter().map(|(_, data)| data).collect()
        }
        else {
            QVariantList::default()
        }
    }

    // Returns map of MonitorData structs in JSON with monitor id as key.
    fn get_monitor_data_map(&self, host_id: QString) -> QVariantMap {
        if let Some(host) = self.hosts.get(&host_id.to_string()) {
            host.monitor_data.clone().data
        }
        else {
            QVariantMap::default()
        }
    }

    // Returns CommandResults from executed commands in JSON.
    // TODO: create a command invocation id to get specific results?
    fn get_command_data(&self, host_id: QString) -> QVariantList {
        if let Some(host) = self.hosts.get(&host_id.to_string()) {
            host.command_data.clone().data
        }
        else {
            QVariantList::default()
        }
    }

    fn get_host_data(&self, index: i32) -> QVariantMap {
        let mut result = QVariantMap::default();

        if let Some(host_id) = self.hosts_index.get(&(index as usize)) {
            let host_data = self.hosts.get(&host_id.to_string()).unwrap();
            result.insert(self.headers[0].clone(), host_data.status.to_qvariant());
            result.insert(self.headers[1].clone(), host_data.name.to_qvariant());
            result.insert(self.headers[2].clone(), host_data.fqdn.to_qvariant());
            result.insert(self.headers[3].clone(), host_data.ip_address.to_qvariant());
        }

        result
    }

    fn get_selected_host(&self) -> QString {
        if let Some(host_id) = self.hosts_index.get(&(self.selected_row as usize)) {
            QString::from(self.hosts.get(host_id).unwrap().name.clone())
        }
        else {
            QString::from("")
        }
    }
   
}


impl QAbstractTableModel for HostListModel {
    fn row_count(&self) -> i32 {
        self.hosts.len() as i32
    }

    fn column_count(&self) -> i32 {
        6
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role != USER_ROLE {
            return QString::from(format!("Unknown role: {}", role)).to_qvariant();
        }

        let host_id = self.hosts_index.get(&(index.row() as usize)).unwrap();
        let row = self.hosts.get(host_id).unwrap();

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