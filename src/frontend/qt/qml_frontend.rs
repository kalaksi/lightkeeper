use std::sync::mpsc;
use std::env;
extern crate qmetaobject;
use qmetaobject::*;
use super::{
    resources,
    models::HostDataManagerModel,
    models::CommandHandlerModel,
    models::PropertyTableModel,
    models::HostTableModel,
    models::ThemeModel,
    models::ConfigManagerModel,
};
use crate::{
    frontend,
    command_handler::CommandHandler,
    monitor_manager::MonitorManager,
    configuration,
    module::Metadata
};


pub struct QmlFrontend {
    theme: Option<ThemeModel>,
    update_sender_prototype: mpsc::Sender<frontend::HostDisplayData>,
    host_data_manager: Option<HostDataManagerModel>,
    command_handler: Option<CommandHandlerModel>,
    config_manager: Option<ConfigManagerModel>,
}

impl QmlFrontend {
    pub fn new(display_data: frontend::DisplayData,
               main_config: configuration::Configuration,
               hosts_config: configuration::Hosts,
               group_config: configuration::Groups,
               module_metadatas: Vec<Metadata>
            ) -> QmlFrontend {
        qmetaobject::log::init_qt_to_rust();
        resources::init_resources();

        let theme_model = ThemeModel::new(main_config.display_options.clone());
        let (host_data_manager, update_sender) = HostDataManagerModel::new(display_data, main_config.clone());
        let config_manager = ConfigManagerModel::new(main_config, hosts_config, group_config, module_metadatas);

        QmlFrontend {
            theme: Some(theme_model),
            update_sender_prototype: update_sender,
            host_data_manager: Some(host_data_manager),
            command_handler: None,
            config_manager: Some(config_manager),
        }
    }

    pub fn setup_command_handler(&mut self, command_handler: CommandHandler, monitor_manager: MonitorManager, display_options: configuration::DisplayOptions) {
        self.command_handler = Some(CommandHandlerModel::new(command_handler, monitor_manager, display_options));
    }

    pub fn start(&mut self) {
        let main_qml_path = if env::var("FLATPAK_ID").is_ok() {
            // Inside flatpak.
            "/app/qml/main.qml"
        }
        else {
            // If running from the source directory, use the QML file from there.
            "src/frontend/qt/qml/main.qml"
        };

        qml_register_type::<PropertyTableModel>(cstr::cstr!("PropertyTableModel"), 1, 0, cstr::cstr!("PropertyTableModel"));
        qml_register_type::<HostTableModel>(cstr::cstr!("HostTableModel"), 1, 0, cstr::cstr!("HostTableModel"));

        let qt_data_theme = QObjectBox::new(self.theme.take().unwrap());
        let qt_data_host_data_manager = QObjectBox::new(self.host_data_manager.take().unwrap());
        let qt_data_command_handler = QObjectBox::new(self.command_handler.take().unwrap());
        let qt_data_config_manager = QObjectBox::new(self.config_manager.take().unwrap());

        let mut engine = QmlEngine::new();
        engine.set_object_property(QString::from("Theme"), qt_data_theme.pinned());
        engine.set_object_property(QString::from("HostDataManager"), qt_data_host_data_manager.pinned());
        engine.set_object_property(QString::from("CommandHandler"), qt_data_command_handler.pinned());
        engine.set_object_property(QString::from("ConfigManager"), qt_data_config_manager.pinned());
        engine.load_file(QString::from(main_qml_path));
        engine.exec();
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::HostDisplayData> {
        self.update_sender_prototype.clone()
    }
}