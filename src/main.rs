#![allow(clippy::redundant_field_names)]
#![allow(clippy::needless_return)]
#![forbid(unsafe_code)]

mod module;
mod host_manager;
mod monitor_manager;
mod host;
mod configuration;
mod utils;
mod enums;
mod frontend;
mod connection_manager;
mod command_handler;
mod file_handler;
mod cache;
pub mod error;

use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

use clap::Parser;

use host_manager::HostManager;
use monitor_manager::MonitorManager;
use connection_manager::ConnectionManager;
use command_handler::CommandHandler;
use host::Host;
use configuration::Configuration;
use module::ModuleFactory;


#[derive(Parser, Clone)]
struct Args {
    #[clap(short, long, default_value = "")]
    config_dir: String,
    #[clap(long)]
    monitoring_module_info: bool,
    #[clap(long)]
    command_module_info: bool,
    #[clap(long)]
    connector_module_info: bool,
}

#[derive(PartialEq)]
pub enum ExitReason {
    Quit,
    Error,
    Restart,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    loop {
        let exit_reason = run(args.clone());
        match exit_reason {
            ExitReason::Quit => break,
            ExitReason::Error => break,
            ExitReason::Restart => continue,
        };
    }
}

fn run(args: Args) -> ExitReason {
    log::info!("Lightkeeper starting...");


    let module_factory = Arc::<ModuleFactory>::new(ModuleFactory::new());
    if args.monitoring_module_info {
        print!("{}", module_factory.get_monitoring_module_info());
        return ExitReason::Quit;
    }
    if args.command_module_info {
        print!("{}", module_factory.get_command_module_info());
        return ExitReason::Quit;
    }
    if args.connector_module_info {
        print!("{}", module_factory.get_connector_module_info());
        return ExitReason::Quit;
    }

    let (main_config, hosts_config, group_config) = match Configuration::read(&args.config_dir) {
        Ok(configuration) => configuration,
        Err(error) => {
            log::error!("Error while reading configuration files: {}", error);
            return ExitReason::Error;
        }
    };

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

    host_manager.borrow_mut().start_receiving_updates();
    connection_manager.start_processing_requests();
    command_handler.start_processing_responses();
    monitor_manager.start_processing_responses();

    let module_metadatas = module_factory.get_module_metadatas();
    let mut frontend = frontend::qt::QmlFrontend::new(
        host_manager.borrow().get_display_data(),
        args.config_dir.clone(),
        main_config.clone(),
        hosts_config.clone(),
        group_config,
        module_metadatas,
    );

    host_manager.borrow_mut().add_observer(frontend.new_update_sender());
    let exit_reason = frontend.start(command_handler, monitor_manager, connection_manager, host_manager, main_config.clone());

    exit_reason
}