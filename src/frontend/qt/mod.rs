mod monitor_data_model;
mod command_data_model;
mod host_list_model;
use host_list_model::HostListModel;
mod command_handler_model;
use command_handler_model::CommandHandlerModel;

mod resources;

use std::sync::mpsc;
extern crate qmetaobject;
use qmetaobject::*;

use crate::{frontend, command_handler::CommandHandler};

pub struct QmlFrontend {
    update_sender_prototype: mpsc::Sender<frontend::HostDisplayData>,
    host_list_model: Option<HostListModel>,
    command_handler_model: Option<CommandHandlerModel>,
}

impl QmlFrontend {
    pub fn new(display_data: &frontend::DisplayData) -> Self {
        qmetaobject::log::init_qt_to_rust();
        resources::init_resources();

        let (host_list_model, update_sender) = HostListModel::new(&display_data);

        QmlFrontend {
            update_sender_prototype: update_sender,
            host_list_model: Some(host_list_model),
            command_handler_model: None,
        }
    }

    pub fn set_command_handler(&mut self, command_handler: CommandHandler) {
        self.command_handler_model = Some(CommandHandlerModel::new(command_handler));
    }

    pub fn start(&mut self) {
        let qt_data_host_list = QObjectBox::new(self.host_list_model.take().unwrap());
        let qt_data_command_methods = QObjectBox::new(self.command_handler_model.take().unwrap());

        let mut engine = QmlEngine::new();
        engine.set_object_property("lightkeeper_data".into(), qt_data_host_list.pinned());
        engine.set_object_property("lightkeeper_commands".into(), qt_data_command_methods.pinned());
        engine.load_file(QString::from("src/frontend/qt/qml/main.qml"));
        engine.exec();
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::HostDisplayData> {
        self.update_sender_prototype.clone()
    }
}
