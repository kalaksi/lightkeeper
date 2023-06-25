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

use std::collections::HashMap;
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


#[derive(Parser)]
#[clap()]
struct Args {
    #[clap(short, long, default_value = "")]
    config_dir: String,
}


fn main() {
    env_logger::init();
    log::info!("Lightkeeper starting...");

    let args = Args::parse();

    let (config, hosts_config) = match Configuration::read(&args.config_dir) {
        Ok(configuration) => configuration,
        Err(error) => {
            log::error!("Error while reading configuration file: {}", error);
            return;
        }
    };

    let module_factory = ModuleFactory::new();

    let host_manager = Rc::new(RefCell::new(HostManager::new()));
    let mut connection_manager = ConnectionManager::new(config.cache_settings.clone());
    let mut monitor_manager = MonitorManager::new(connection_manager.new_request_sender(), host_manager.clone(), config.cache_settings.clone());
    let mut command_handler = CommandHandler::new(&config.preferences, connection_manager.new_request_sender(), host_manager.clone());
    host_manager.borrow_mut().start_receiving_updates();

    // Configure hosts and modules.
    for (host_id, host_config) in hosts_config.hosts.iter() {
        log::info!("Found configuration for host {}", host_id);

        let host = match Host::new(&host_id, &host_config.address, &host_config.fqdn, &host_config.settings.clone().unwrap_or_default()) {
            Ok(host) => host,
            Err(error) => {
                log::error!("{}", error);
                continue;
            }
        };

        if let Err(error) = host_manager.borrow_mut().add_host(host.clone()) {
            log::error!("{}", error.to_string());
            continue;
        };

        for (monitor_id, monitor_config) in host_config.monitors.iter() {
            let monitor_spec = ModuleSpecification::new(monitor_id.as_str(), monitor_config.version.as_str());
            let monitor = module_factory.new_monitor(&monitor_spec, &monitor_config.settings);

            // Initialize a connector if the monitor uses any.
            if let Some(connector_spec) = monitor.get_connector_spec() {
                let connector_settings = match host_config.connectors.get(&connector_spec.id) {
                    Some(config) => config.settings.clone(),
                    None => HashMap::new(),
                };

                let connector = module_factory.new_connector(&connector_spec, &connector_settings);
                connection_manager.add_connector(&host, connector);
            }

            monitor_manager.add_monitor(&host, monitor);
        }

        for (command_id, command_config) in host_config.commands.iter() {
            let command_spec = ModuleSpecification::new(command_id.as_str(), command_config.version.as_str());
            let command = module_factory.new_command(&command_spec, &command_config.settings);

            if let Some(connector_spec) = command.get_connector_spec() {
                let connector_settings = match host_config.connectors.get(&connector_spec.id) {
                    Some(config) => config.settings.clone(),
                    None => HashMap::new(),
                };
                let connector = module_factory.new_connector(&connector_spec, &connector_settings);
                connection_manager.add_connector(&host, connector);
            }

            command_handler.add_command(&host.name, command);
        }
    }

    connection_manager.start(module_factory);

    let mut initial_display_data = host_manager.borrow().get_display_data();
    initial_display_data.table_headers = vec![String::from("Status"), String::from("Name"), String::from("FQDN"), String::from("IP address")];

    let mut frontend = frontend::qt::QmlFrontend::new(initial_display_data, config.clone());

    host_manager.borrow_mut().add_observer(frontend.new_update_sender());
    frontend.setup_command_handler(command_handler, monitor_manager, config.display_options.clone());
    frontend.start();

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
}