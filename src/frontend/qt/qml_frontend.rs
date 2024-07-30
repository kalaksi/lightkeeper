use std::rc::Rc;
use std::{sync::mpsc, cell::RefCell};
use std::env;
extern crate qmetaobject;
use qmetaobject::*;
use super::resources;
#[allow(unused_imports)]
use super::resources_qml;
use super::models::*;
use crate::{
    frontend,
    command_handler::CommandHandler,
    monitor_manager::MonitorManager,
    configuration,
    module::Metadata,
    ExitReason,
    connection_manager::ConnectionManager, host_manager,
};


pub struct QmlFrontend {
    theme: Option<ThemeModel>,
    update_sender_prototype: mpsc::Sender<frontend::HostDisplayData>,
    host_data_manager: Option<HostDataManagerModel>,
    config_manager: Option<ConfigManagerModel>,
}

impl QmlFrontend {
    /// Parameters provide the initial data and configuration for the frontend.
    pub fn new(display_data: frontend::DisplayData,
               config_dir: &String,
               main_config: &configuration::Configuration,
               hosts_config: &configuration::Hosts,
               group_config: &configuration::Groups,
               module_metadatas: Vec<Metadata>
            ) -> QmlFrontend {

        qmetaobject::log::init_qt_to_rust();
        resources::init_resources();

        let style = main_config.display_options.qtquick_style.as_str();
        if !style.is_empty() &&
           std::env::var("QT_QUICK_CONTROLS_STYLE").is_err() &&
           std::env::var("QT_STYLE_OVERRIDE").is_err() {

            std::env::set_var("QT_STYLE_OVERRIDE", style);
        }

        let theme_model = ThemeModel::new(main_config.display_options.clone());
        let host_data_manager = HostDataManagerModel::new(display_data, main_config.clone());
        let config_manager = ConfigManagerModel::new(config_dir.clone(), main_config.clone(), hosts_config.clone(), group_config.clone(), module_metadatas);

        QmlFrontend {
            theme: Some(theme_model),
            update_sender_prototype: host_data_manager.new_update_sender(),
            host_data_manager: Some(host_data_manager),
            config_manager: Some(config_manager),
        }
    }

    /// Takes ownership of most components (excl. HostDataManager).
    pub fn start(&mut self,
        command_handler: CommandHandler,
        monitor_manager: MonitorManager,
        connection_manager: ConnectionManager,
        host_manager: Rc<RefCell<host_manager::HostManager>>,
        config: configuration::Configuration) -> ExitReason {

        let sandboxed = env::var("FLATPAK_ID").is_ok();

        let command_handler_model = CommandHandlerModel::new(command_handler, monitor_manager, connection_manager, host_manager, config);

        qml_register_type::<PropertyTableModel>(cstr::cstr!("PropertyTableModel"), 1, 0, cstr::cstr!("PropertyTableModel"));
        qml_register_type::<HostTableModel>(cstr::cstr!("HostTableModel"), 1, 0, cstr::cstr!("HostTableModel"));
        qml_register_type::<CooldownTimerModel>(cstr::cstr!("CooldownTimerModel"), 1, 0, cstr::cstr!("CooldownTimerModel"));

        let qt_data_theme = QObjectBox::new(self.theme.take().unwrap());
        let qt_data_host_data_manager = QObjectBox::new(self.host_data_manager.take().unwrap());
        let qt_data_command_handler = QObjectBox::new(command_handler_model);
        let qt_data_desktop_portal = QObjectBox::new(DesktopPortalModel::new());
        let qt_data_config_manager = QObjectBox::new(self.config_manager.take().unwrap());
        let sandboxed_updated = qt_data_config_manager.pinned().borrow_mut().setSandboxed(sandboxed);
        let mut engine = QmlEngine::new();

        if sandboxed_updated {
            // Currently needs a restart so configuration is updated everywhere. Should happen only on first start.
            return ExitReason::Restart;
        }
        else {
            if sandboxed {
                engine.add_import_path(QString::from("/app/qmltermwidget/usr/lib/qml/"));
            }
            engine.set_object_property(QString::from("Theme"), qt_data_theme.pinned());
            engine.set_object_property(QString::from("HostDataManager"), qt_data_host_data_manager.pinned());
            engine.set_object_property(QString::from("CommandHandler"), qt_data_command_handler.pinned());
            engine.set_object_property(QString::from("ConfigManager"), qt_data_config_manager.pinned());
            engine.set_object_property(QString::from("DesktopPortal"), qt_data_desktop_portal.pinned());
            self.load_qml(&mut engine);
            engine.exec();
        }

        ExitReason::Quit
    }

    #[cfg(debug_assertions)]
    /// Only available in dev build.
    pub fn start_testing(&mut self,
        command_handler: CommandHandler,
        monitor_manager: MonitorManager,
        connection_manager: ConnectionManager,
        host_manager: Rc<RefCell<host_manager::HostManager>>,
        config: configuration::Configuration) -> QmlEngine {

        let command_handler_model = CommandHandlerModel::new(command_handler, monitor_manager, connection_manager, host_manager, config);

        qml_register_type::<PropertyTableModel>(cstr::cstr!("PropertyTableModel"), 1, 0, cstr::cstr!("PropertyTableModel"));
        qml_register_type::<HostTableModel>(cstr::cstr!("HostTableModel"), 1, 0, cstr::cstr!("HostTableModel"));
        qml_register_type::<CooldownTimerModel>(cstr::cstr!("CooldownTimerModel"), 1, 0, cstr::cstr!("CooldownTimerModel"));

        let qt_data_theme = QObjectBox::new(self.theme.take().unwrap());
        let qt_data_host_data_manager = QObjectBox::new(self.host_data_manager.take().unwrap());
        let qt_data_command_handler = QObjectBox::new(command_handler_model);
        let qt_data_desktop_portal = QObjectBox::new(DesktopPortalModel::new());
        let qt_data_config_manager = QObjectBox::new(self.config_manager.take().unwrap());

        let mut engine = QmlEngine::new();
        engine.set_object_property(QString::from("Theme"), qt_data_theme.pinned());
        engine.set_object_property(QString::from("HostDataManager"), qt_data_host_data_manager.pinned());
        engine.set_object_property(QString::from("CommandHandler"), qt_data_command_handler.pinned());
        engine.set_object_property(QString::from("ConfigManager"), qt_data_config_manager.pinned());
        engine.set_object_property(QString::from("DesktopPortal"), qt_data_desktop_portal.pinned());
        self.load_qml(&mut engine);
        engine
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::HostDisplayData> {
        self.update_sender_prototype.clone()
    }

    // In development, using file paths helps avoid recompilation when only QML changes.
    #[cfg(debug_assertions)]
    fn load_qml(&self, engine: &mut QmlEngine) {
        engine.load_file(QString::from("src/frontend/qt/qml/Main.qml"));
    }

    #[cfg(not(debug_assertions))]
    fn load_qml(&self, engine: &mut QmlEngine) {
        resources_qml::init_resources();
        engine.load_url(QUrl::from(QString::from("qrc:/qml/Main.qml")));
    }
}