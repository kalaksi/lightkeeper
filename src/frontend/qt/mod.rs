use std::str::FromStr;
use std::thread;
use std::sync::mpsc;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use crate::module::monitoring::{
    Criticality,
    DataPoint,
    DisplayOptions,
    DisplayStyle
};

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
    base: qt_base_class!(trait QAbstractListModel),
    headers: Vec<QString>,
    hosts: qt_property!(HostCollection; NOTIFY hosts_changed),
    hosts_changed: qt_signal!(),

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
            let qptr = QPointer::from(&*self);
            let set_value = qmetaobject::queued_callback(move |host_data: HostData| {
                qptr.as_pinned().map(|self_| {
                    let _old_value = std::mem::replace(
                        self_.borrow_mut().hosts.data.iter_mut().find(|host| host.name == host_data.name).unwrap(),
                        host_data,
                    );
                    self_.borrow().hosts_changed();
                });
            });

            let receiver = self.update_receiver.take().unwrap();
            let thread = std::thread::spawn(move || {
                loop {
                    let received_data = receiver.recv().unwrap();
                    let host_data = HostData::from(&received_data);
                    set_value(host_data);
                }
            });

            self.update_receiver_thread = Some(thread);
        }
    }
}


impl QAbstractListModel for HostList {
    fn row_count(&self) -> i32 {
        self.hosts.data.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role != USER_ROLE {
            return QVariant::default();
        }

        self.hosts.data.get(index.row() as usize).map(|item| item.to_qvariant()).unwrap_or_default()
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
    monitor_data: Vec<QString>,
}

impl HostData {
    pub fn from(host_display_data: &frontend::HostDisplayData) -> Self {
        let mut monitor_data: Vec<QString> = host_display_data.monitoring_data.iter().map(|(_, data)| {
                convert_to_display_string(data.values.last().unwrap(), &data.display_options)
        }).collect();

        HostData {
            status: host_display_data.status.clone().to_string().into(),
            name: host_display_data.name.clone().into(),
            fqdn: host_display_data.domain_name.clone().into(),
            ip_address: host_display_data.ip_address.to_string().into(),
            monitor_data: monitor_data,
        }
    }

}

fn convert_to_display_string(data_point: &DataPoint, display_options: &DisplayOptions) -> QString {
    if data_point.is_empty() {
        if data_point.criticality == Criticality::Critical {
            QString::from("Error")
        }
        else {
            QString::from("TODO")
        }
    }
    else if display_options.use_multivalue {
        convert_multivalue(data_point, display_options)
    }
    else {
        convert_single(data_point, display_options)
    }
}

fn convert_single(data_point: &DataPoint, display_options: &DisplayOptions) -> QString {
    let single_value = match display_options.display_style {
        DisplayStyle::CriticalityLevel => {
            match data_point.criticality {
                Criticality::NoData => QString::from("No data"),
                Criticality::Normal => QString::from("Normal"),
                Criticality::Warning => QString::from("Warning"),
                Criticality::Error => QString::from("Error"),
                Criticality::Critical => QString::from("Critical"),
            }
        },
        DisplayStyle::StatusUpDown => {
            match crate::utils::enums::HostStatus::from_str(&data_point.value).unwrap_or_default() {
                crate::utils::enums::HostStatus::Up => QString::from("Up"),
                crate::utils::enums::HostStatus::Down => QString::from("Down"),
            }
        },
        DisplayStyle::String => {
            QString::from(data_point.value.clone())
        },
    };

    single_value
}

fn convert_multivalue(data_point: &DataPoint, display_options: &DisplayOptions) -> QString {
    let mut separator = ", ";

    // Process all values and join them into string in the end.
    let display_value = data_point.multivalue.iter().map(|data_point| {
        let single_value = match display_options.display_style {
            DisplayStyle::CriticalityLevel => {
                separator = "";

                match data_point.criticality {
                    Criticality::NoData => "No data".to_string(),
                    Criticality::Normal => "▩".to_string(),
                    Criticality::Warning =>"▩".to_string(),
                    Criticality::Error => "▩".to_string(),
                    Criticality::Critical =>"▩".to_string(),
                }
            },
            DisplayStyle::StatusUpDown => {
                match crate::utils::enums::HostStatus::from_str(&data_point.value).unwrap_or_default() {
                    crate::utils::enums::HostStatus::Up => String::from("Up"),
                    crate::utils::enums::HostStatus::Down => String::from("Down"),
                }
            },
            DisplayStyle::String => {
                data_point.value.to_string()
            },
        };

        single_value
    }).collect::<Vec<String>>();

    QString::from(display_value.join(separator))
}