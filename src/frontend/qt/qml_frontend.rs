
use std::sync::mpsc;
extern crate qmetaobject;
use qmetaobject::*;
use super::{
    resources,
    models::HostDataManagerModel,
    models::CommandHandlerModel,
    models::PropertyTableModel,
    models::HostTableModel,
    models::ThemeModel,
};
use crate::{
    frontend,
    command_handler::CommandHandler,
    monitor_manager::MonitorManager,
    configuration
};


pub struct QmlFrontend {
    theme: Option<ThemeModel>,
    update_sender_prototype: mpsc::Sender<frontend::HostDisplayData>,
    host_data_manager: Option<HostDataManagerModel>,
    command_handler: Option<CommandHandlerModel>,
}

impl QmlFrontend {
    pub fn new(display_data: frontend::DisplayData, config: configuration::Configuration) -> Self {
        qmetaobject::log::init_qt_to_rust();
        resources::init_resources();

        let theme_model = ThemeModel::new(config.display_options.clone());
        let (host_data_manager, update_sender) = HostDataManagerModel::new(display_data, config);

        QmlFrontend {
            theme: Some(theme_model),
            update_sender_prototype: update_sender,
            host_data_manager: Some(host_data_manager),
            command_handler: None,
        }
    }

    pub fn setup_command_handler(&mut self, command_handler: CommandHandler, monitor_manager: MonitorManager, display_options: configuration::DisplayOptions) {
        self.command_handler = Some(CommandHandlerModel::new(command_handler, monitor_manager, display_options));
    }

    pub fn start(&mut self) {
        qml_register_type::<PropertyTableModel>(cstr::cstr!("PropertyTableModel"), 1, 0, cstr::cstr!("PropertyTableModel"));
        qml_register_type::<HostTableModel>(cstr::cstr!("HostTableModel"), 1, 0, cstr::cstr!("HostTableModel"));

        let qt_data_theme = QObjectBox::new(self.theme.take().unwrap());
        let qt_data_host_data_manager = QObjectBox::new(self.host_data_manager.take().unwrap());
        let qt_data_command_handler = QObjectBox::new(self.command_handler.take().unwrap());

        let mut engine = QmlEngine::new();
        engine.set_object_property(QString::from("Theme"), qt_data_theme.pinned());
        engine.set_object_property(QString::from("HostDataManager"), qt_data_host_data_manager.pinned());
        engine.set_object_property(QString::from("CommandHandler"), qt_data_command_handler.pinned());
        engine.load_file(QString::from("src/frontend/qt/qml/main.qml"));
        engine.exec();
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::HostDisplayData> {
        self.update_sender_prototype.clone()
    }
}