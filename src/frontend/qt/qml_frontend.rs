use std::rc::Rc;
use std::{sync::mpsc, cell::RefCell};
use std::env;
extern crate qmetaobject;
use qmetaobject::*;
use super::resources;
#[allow(unused_imports)]
use super::resources_qml;
use super::models::*;
use crate::metrics::MetricsManager;
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
    config_dir: String,
    main_config: configuration::Configuration,
    hosts_config: configuration::Hosts,
    group_config: configuration::Groups,
    module_metadatas: Vec<Metadata>,
    update_receiver: Option<mpsc::Receiver<frontend::UIUpdate>>,
    update_sender_prototype: mpsc::Sender<frontend::UIUpdate>,
}

impl QmlFrontend {
    /// Parameters provide the initial data and configuration for the frontend.
    pub fn new(config_dir: &String,
               main_config: &configuration::Configuration,
               hosts_config: &configuration::Hosts,
               group_config: &configuration::Groups,
               module_metadatas: Vec<Metadata>) -> QmlFrontend {

        qmetaobject::log::init_qt_to_rust();
        resources::init_resources();

        let style = main_config.display_options.qtquick_style.as_str();
        if !style.is_empty() &&
           std::env::var("QT_QUICK_CONTROLS_STYLE").is_err() &&
           std::env::var("QT_STYLE_OVERRIDE").is_err() {

            std::env::set_var("QT_STYLE_OVERRIDE", style);
        }

        let (sender, receiver) = mpsc::channel::<frontend::UIUpdate>();

        QmlFrontend {
            main_config: main_config.clone(),
            config_dir: config_dir.clone(),
            hosts_config: hosts_config.clone(),
            group_config: group_config.clone(),
            module_metadatas: module_metadatas,
            update_receiver: Some(receiver),
            update_sender_prototype: sender,
        }
    }

    /// Takes ownership of most components (excl. HostDataManager).
    pub fn start(&mut self,
        command_handler: CommandHandler,
        monitor_manager: MonitorManager,
        connection_manager: ConnectionManager,
        host_manager: Rc<RefCell<host_manager::HostManager>>,
        metrics_manager: Option<MetricsManager>) -> ExitReason {

        qml_register_type::<PropertyTableModel>(cstr::cstr!("PropertyTableModel"), 1, 0, cstr::cstr!("PropertyTableModel"));
        qml_register_type::<HostTableModel>(cstr::cstr!("HostTableModel"), 1, 0, cstr::cstr!("HostTableModel"));

        let display_data = host_manager.borrow().get_display_data();
        let qt_theme = QObjectBox::new(ThemeModel::new(self.main_config.display_options.clone()));
        let qt_file_chooser = QObjectBox::new(FileChooserModel::new());
        let qt_lkbackend = QObjectBox::new(LkBackend::new(
            self.update_sender_prototype.clone(),
            self.update_receiver.take().unwrap(),
            host_manager,
            connection_manager,
            HostDataManagerModel::new(display_data, self.main_config.clone()),
            CommandHandlerModel::new(command_handler, monitor_manager, self.main_config.clone()),
            MetricsManagerModel::new(metrics_manager),
            ConfigManagerModel::new(self.config_dir.clone(), self.main_config.clone(), self.hosts_config.clone(), self.group_config.clone(), self.module_metadatas.clone()),
        ));

        let sandboxed = env::var("FLATPAK_ID").is_ok();
        let sandboxed_updated = qt_lkbackend.pinned().borrow_mut().config.borrow_mut().setSandboxed(sandboxed);
        let mut engine = QmlEngine::new();

        if sandboxed_updated {
            // Currently needs a restart so configuration is updated everywhere. Should happen only on first start.
            return ExitReason::Restart;
        }
        else {
            if sandboxed {
                engine.add_import_path(QString::from("/app/qmltermwidget/usr/qml/"));
                engine.add_import_path(QString::from("/app/ChartJs2QML"));
            }
            else {
                engine.add_import_path(QString::from("./third_party/qmltermwidget"));
                engine.add_import_path(QString::from("./third_party/ChartJs2QML"));
            }
            engine.set_object_property(QString::from("LK"), qt_lkbackend.pinned());
            engine.set_object_property(QString::from("Theme"), qt_theme.pinned());
            engine.set_object_property(QString::from("DesktopPortal"), qt_file_chooser.pinned());
            ::log::debug!("Temporary log entry 8");
            self.load_qml(&mut engine);
            ::log::debug!("Temporary log entry 9");
            engine.exec();
        }

        ExitReason::Quit
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::UIUpdate> {
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