
use super::{
    resources,
    models::HostDataManagerModel,
    models::CommandHandlerModel,
    models::HostTableModel,
};

use std::sync::mpsc;
extern crate qmetaobject;
use qmetaobject::*;
use crate::{frontend, command_handler::CommandHandler};

pub struct QmlFrontend {
    update_sender_prototype: mpsc::Sender<frontend::HostDisplayData>,
    host_data_manager: Option<HostDataManagerModel>,
    command_handler: Option<CommandHandlerModel>,
    host_table: Option<HostTableModel>,
}

impl QmlFrontend {
    pub fn new(display_data: &frontend::DisplayData) -> Self {
        qmetaobject::log::init_qt_to_rust();
        resources::init_resources();

        let (host_data_manager, update_sender) = HostDataManagerModel::new(&display_data);

        QmlFrontend {
            update_sender_prototype: update_sender,
            host_data_manager: Some(host_data_manager),
            command_handler: None,
            host_table: Some(HostTableModel::new(&display_data)),
        }
    }

    pub fn set_command_handler(&mut self, command_handler: CommandHandler) {
        self.command_handler = Some(CommandHandlerModel::new(command_handler));
    }

    pub fn start(&mut self) {
        let qt_data_host_data_manager = QObjectBox::new(self.host_data_manager.take().unwrap());
        let qt_data_command_handler = QObjectBox::new(self.command_handler.take().unwrap());
        let qt_data_host_table = QObjectBox::new(self.host_table.take().unwrap());

        let mut engine = QmlEngine::new();
        engine.set_object_property("_hostDataManager".into(), qt_data_host_data_manager.pinned());
        engine.set_object_property("_commandHandler".into(), qt_data_command_handler.pinned());
        engine.set_object_property("_hostTableModel".into(), qt_data_host_table.pinned());
        engine.load_file(QString::from("src/frontend/qt/qml/main.qml"));
        engine.exec();
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::HostDisplayData> {
        self.update_sender_prototype.clone()
    }
}