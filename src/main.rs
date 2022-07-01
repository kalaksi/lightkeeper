mod module;
mod host_manager;
mod monitor_manager;
mod host;
mod configuration;
mod utils;
mod frontend;
mod connection_manager;
mod command_handler;

use std::collections::HashMap;

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
    #[clap(short, long, default_value = "config.yml")]
    config_file: String,
}


fn main() {
    env_logger::init();
    log::info!("Lightkeeper starting...");

    let args = Args::parse();

    let config = match Configuration::read(&args.config_file) {
        Ok(configuration) => configuration,
        Err(error) => {
            log::error!("Error while reading configuration file: {}", error);
            return;
        }
    };

    let module_factory = ModuleFactory::new();

    let mut host_manager = HostManager::new();
    let mut connection_manager = ConnectionManager::new();
    let mut monitor_manager = MonitorManager::new(connection_manager.new_request_sender(), host_manager.new_state_update_sender());
    let command_handler = CommandHandler::new(connection_manager.new_request_sender(), host_manager.new_state_update_sender());

    // Configure hosts and modules.
    for (host_id, host_config) in config.hosts.iter() {
        log::info!("Found configuration for host {}", host_id);

        let host = match Host::new(&host_id, &host_config.address, &host_config.fqdn) {
            Ok(host) => host,
            Err(error) => {
                log::error!("{}", error);
                continue;
            }
        };

        if let Err(error) = host_manager.add_host(host.clone(), config.general.default_host_status) {
            log::error!("{}", error.to_string());
            continue;
        };

        for (monitor_id, monitor_config) in host_config.monitors.iter() {
            let monitor_spec = ModuleSpecification::new(monitor_id.as_str(), monitor_config.version.as_str());
            let monitor = module_factory.new_monitor(&monitor_spec, &monitor_config.settings);

            // Initialize a connector if the monitors uses any.
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
        }
    }

    monitor_manager.refresh_monitors();
    let initial_display_data = host_manager.get_display_data();

    command_handler.execute(
        host_manager.get_host(&String::from("test11")),
        module_factory.new_command(&ModuleSpecification::new("docker", "0.0.1"), &HashMap::new())
    );

    let mut frontend = frontend::qt::QmlFrontend::new(&initial_display_data);
    host_manager.add_observer(frontend.new_update_sender());

    frontend.start();

    connection_manager.join();
    host_manager.join();

}