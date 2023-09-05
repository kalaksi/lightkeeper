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

use std::{collections::HashMap, sync::Arc};
use std::cell::RefCell;
use std::rc::Rc;

use clap::Parser;

use host_manager::HostManager;
use monitor_manager::MonitorManager;
use connection_manager::ConnectionManager;
use command_handler::CommandHandler;
use host::Host;
use configuration::Configuration;
use module::{ ModuleFactory, ModuleSpecification };


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
    let mut connection_manager = ConnectionManager::new(main_config.cache_settings.clone());
    let mut monitor_manager = MonitorManager::new(connection_manager.new_request_sender(), main_config.cache_settings.clone(), host_manager.clone(), module_factory.clone());
    let mut command_handler = CommandHandler::new(connection_manager.new_request_sender(), host_manager.clone(), module_factory.clone());
    host_manager.borrow_mut().start_receiving_updates();

    // Configure hosts and modules.
    host_manager.borrow_mut().configure(&hosts_config);
    monitor_manager.configure(&hosts_config, &mut connection_manager);
    command_handler.configure(&hosts_config, &main_config.preferences, &mut connection_manager);

    let module_metadatas = module_factory.get_module_metadatas();
    connection_manager.start(module_factory);

    let mut frontend = frontend::qt::QmlFrontend::new(
        host_manager.borrow().get_display_data(),
        args.config_dir.clone(),
        main_config.clone(),
        hosts_config.clone(),
        group_config,
        module_metadatas,
    );

    host_manager.borrow_mut().add_observer(frontend.new_update_sender());
    frontend.setup_command_handler(command_handler, monitor_manager, main_config.display_options.clone().unwrap());
    let exit_reason = frontend.start();

    // Shut down threads.
    connection_manager.new_request_sender()
                      .send(connection_manager::ConnectorRequest::exit_token())
                      .unwrap_or_else(|error| log::error!("Couldn't send exit token to connection manager: {}", error));
    connection_manager.join();

    host_manager.borrow_mut()
                .new_state_update_sender()
                .send(host_manager::StateUpdateMessage::exit_token())
                .unwrap_or_else(|error| log::error!("Couldn't send exit token to state manager: {}", error));
    host_manager.borrow_mut().join();

    exit_reason
}