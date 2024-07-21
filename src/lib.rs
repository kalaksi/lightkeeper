
extern crate qmetaobject;
use qmetaobject::*;

pub mod error;
pub mod module;
pub mod configuration;
mod host_manager;
mod monitor_manager;
mod host;
mod utils;
mod enums;
mod frontend;
mod connection_manager;
mod command_handler;
mod file_handler;
mod cache;

pub use module::ModuleFactory;
pub use configuration::Configuration;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use clap::Parser;
use host_manager::HostManager;
use monitor_manager::MonitorManager;
use connection_manager::ConnectionManager;
use command_handler::CommandHandler;
use host::Host;


#[derive(Parser, Clone)]
pub struct Args {
    #[clap(short, long, default_value = "")]
    pub config_dir: String,
    #[clap(long)]
    pub monitoring_module_info: bool,
    #[clap(long)]
    pub command_module_info: bool,
    #[clap(long)]
    pub connector_module_info: bool,
}

#[derive(PartialEq)]
pub enum ExitReason {
    Quit,
    Error,
    Restart,
}

pub fn run(
    config_dir: &String,
    main_config: &Configuration,
    hosts_config: &configuration::Hosts,
    group_config: &configuration::Groups,
    test: bool) -> (ExitReason, QmlEngine) {

    let module_factory = Arc::<ModuleFactory>::new(ModuleFactory::new());

    let host_manager = Rc::new(RefCell::new(HostManager::new()));
    host_manager.borrow_mut().configure(&hosts_config);

    let mut connection_manager = ConnectionManager::new(module_factory.clone());
    connection_manager.configure(&hosts_config, &main_config.cache_settings);

    let mut monitor_manager = MonitorManager::new(main_config.cache_settings.clone(), host_manager.clone(), module_factory.clone());
    monitor_manager.configure(
        &hosts_config,
        connection_manager.new_request_sender(),
        host_manager.borrow().new_state_update_sender()
    );

    let mut command_handler = CommandHandler::new(host_manager.clone(), module_factory.clone());
    command_handler.configure(
        &hosts_config,
        &main_config.preferences,
        connection_manager.new_request_sender(),
        host_manager.borrow().new_state_update_sender()
    );

    // Start backend threads.
    host_manager.borrow_mut().start_receiving_updates();
    connection_manager.start_processing_requests();
    command_handler.start_processing_responses();
    monitor_manager.start_processing_responses();

    // TODO: Needs a proper fix for a race.
    // Wait a small amount as a workaround so initial data points have time to get sent to host manager.
    // Otherwise, initial status summary icons are randomly not shown.
    std::thread::sleep(std::time::Duration::from_millis(100));

    let module_metadatas = module_factory.get_module_metadatas();
    let mut frontend = frontend::qt::QmlFrontend::new(
        host_manager.borrow().get_display_data(),
        config_dir,
        main_config,
        hosts_config,
        group_config,
        module_metadatas,
    );

    host_manager.borrow_mut().add_observer(frontend.new_update_sender());
    if test {
        let qml_engine = frontend.start_testing(command_handler, monitor_manager, connection_manager, host_manager, main_config.clone());
        (ExitReason::Quit, qml_engine)
    }
    else {
        let (exit_reason, qml_engine) = frontend.start(command_handler, monitor_manager, connection_manager, host_manager, main_config.clone());
        (exit_reason, qml_engine)
    }
}