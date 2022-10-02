use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::command_data_model::CommandDataModel;
use super::monitor_data_model::MonitorDataModel;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
pub struct HostDataManagerModel {
    base: qt_base_class!(trait QObject),
    hosts: HashMap<String, HostData>,
    hosts_index: HashMap<usize, String>,

    receive_updates: qt_method!(fn(&self)),
    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,

    // NOTE: Couldn't get custom types to work for return types,
    // so for now methods are used to get the data in JSON and parsed in QML side.
    get_monitor_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_monitor_data_map: qt_method!(fn(&self, host_id: QString) -> QVariantMap),
    get_command_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_host_data: qt_method!(fn(&self, index: i32) -> QVariantMap),
}

impl HostDataManagerModel {
    pub fn new(display_data: &frontend::DisplayData) -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let (sender, receiver) = mpsc::channel::<frontend::HostDisplayData>();
        let mut model = HostListModel {
            hosts: HashMap::new(),
            hosts_index: HashMap::new(),
            update_receiver: Some(receiver),
            update_receiver_thread: None,
            selected_row: -1,
            ..Default::default()
        };

        for (host_id, host_data) in display_data.hosts.iter() {
            model.hosts_index.insert(model.hosts.len(), host_id.clone()); model.hosts.insert(host_id.clone(), HostData::from(&host_data));
        }

        (model, sender)
    }

    fn receive_updates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        if self.update_receiver_thread.is_none() {
            let self_ptr = QPointer::from(&*self);
            let set_data = qmetaobject::queued_callback(move |host_display_data: frontend::HostDisplayData| {
                self_ptr.as_pinned().map(|self_pinned| {
                    // HostData cannot be passed between threads so parsing happens in set_data().
                    let host_data = HostData::from(&host_display_data);

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
}