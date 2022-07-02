mod monitor_data_model;
mod command_data_model;
mod host_list_model;
use host_list_model::HostListModel;

mod resources;

use std::sync::mpsc;
extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::DisplayData;

pub struct QmlFrontend {
    update_sender_prototype: mpsc::Sender<frontend::HostDisplayData>,
    model: Option<HostListModel>,
}

impl QmlFrontend {
    pub fn new(display_data: &DisplayData) -> Self {
        qmetaobject::log::init_qt_to_rust();
        resources::init_resources();

        let (data_model, update_sender) = HostListModel::new(&display_data);

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
