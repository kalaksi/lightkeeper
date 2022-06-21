use std::thread;
use std::sync::mpsc;

use serde_json;
extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::DisplayData;

pub struct QmlFrontend {
    update_sender_prototype: mpsc::Sender<frontend::HostDisplayData>,
    model: Option<HostList>,
}

impl QmlFrontend {
    pub fn new(display_data: &DisplayData) -> Self {
        qmetaobject::log::init_qt_to_rust();

        let (data_model, update_sender) = HostList::from(&display_data);

        QmlFrontend {
            update_sender_prototype: update_sender,
            model: Some(data_model),
        }
    }

    pub fn start(&mut self) {
        let qt_data = QObjectBox::new(self.model.take().unwrap());
        let mut engine = QmlEngine::new();
        engine.set_object_property("lightkeeper_data".into(), qt_data.pinned());

        engine.load_file(QString::from("src/frontend/qt/qml/main.qml"));
        engine.exec();
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::HostDisplayData> {
        self.update_sender_prototype.clone()
    }
}


#[derive(QObject, Default)]
struct HostList {
    base: qt_base_class!(trait QAbstractTableModel),
    headers: Vec<QString>,
    hosts: qt_property!(HostCollection),

    receive_updates: qt_method!(fn(&self)),
    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,
}

impl HostList {
    pub fn new() -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let (sender, receiver) = mpsc::channel::<frontend::HostDisplayData>();
        let model = HostList {
            headers: Vec::new(),
            hosts: HostCollection::new(),
            update_receiver: Some(receiver),
            update_receiver_thread: None,
            ..Default::default()
        };

        (model, sender)
    }

    pub fn from(display_data: &frontend::DisplayData) -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let (mut model, update_sender)= HostList::new();

        for header in &display_data.table_headers {
            model.headers.push(header.clone().into());
        }

        for (_, host_data) in display_data.hosts.iter() {
            model.hosts.data.push(HostData::from(&host_data))
        }

        (model, update_sender)
    }

    fn receive_updates(&mut self) {
        // Shouldn't be run more than once.
        if self.update_receiver_thread.is_none() {
            let self_ptr = QPointer::from(&*self);
            let set_data = qmetaobject::queued_callback(move |host_data: HostData| {
                self_ptr.as_pinned().map(|self_pinned| {
                    let _old_value = std::mem::replace(
                        self_pinned.borrow_mut().hosts.data.iter_mut().find(|host| host.name == host_data.name).unwrap(),
                        host_data,
                    );

                    // TODO:
                    // let index = self_pinned.borrow().hosts.data.iter().position(|&item| item.name == host_data.name).unwrap();
                    let top_left = self_pinned.borrow().index(0, 0);
                    let bottom_right = self_pinned.borrow().index(
                        self_pinned.borrow().hosts.data.len() as i32 - 1,
                        self_pinned.borrow().column_count() as i32 - 1
                    );
                    self_pinned.borrow_mut().data_changed(top_left, bottom_right);
                });
            });

            let receiver = self.update_receiver.take().unwrap();
            let thread = std::thread::spawn(move || {
                loop {
                    let received_data = receiver.recv().unwrap();
                    let host_data = HostData::from(&received_data);
                    set_data(host_data);
                }
            });

            self.update_receiver_thread = Some(thread);
        }
    }
}


impl QAbstractTableModel for HostList {
    fn row_count(&self) -> i32 {
        self.hosts.data.len() as i32
    }

    fn column_count(&self) -> i32 {
        5
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role != USER_ROLE {
            return QVariant::default();
        }

        let row = self.hosts.data.get(index.row() as usize).unwrap();

        match index.column() {
            0 => row.status.to_qvariant(),
            1 => row.name.to_qvariant(),
            2 => row.fqdn.to_qvariant(),
            3 => row.ip_address.to_qvariant(),
            _ => {
                row.monitor_data_json.to_qvariant()
            }
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}

#[derive(QGadget, Default, Clone)]
struct HostCollection {
    // TODO: dictionary or similar 
    data: Vec<HostData>,
}

impl HostCollection {
    pub fn new() -> Self {
        HostCollection { 
            data: Vec::new()
        }
    }
}

#[derive(QGadget, Default, Clone)]
struct HostData {
    status: qt_property!(QString),
    name: qt_property!(QString),
    fqdn: qt_property!(QString),
    ip_address: qt_property!(QString),
    monitor_data_json: qt_property!(QString),
}

impl HostData {
    pub fn from(host_display_data: &frontend::HostDisplayData) -> Self {
        HostData {
            status: host_display_data.status.clone().to_string().into(),
            name: host_display_data.name.clone().into(),
            fqdn: host_display_data.domain_name.clone().into(),
            ip_address: host_display_data.ip_address.to_string().into(),
            monitor_data_json: serde_json::to_string(&host_display_data.monitoring_data).unwrap().into(),
        }
    }

}