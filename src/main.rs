mod module;
mod host_manager;
mod monitor_manager;
mod host;
mod configuration;
mod utils;
mod frontend;
mod connection_manager;

use std::collections::HashMap;

use clap::Parser;

use host_manager::HostManager;
use monitor_manager::MonitorManager;
use connection_manager::ConnectionManager;
use host::Host;
use configuration::Configuration;
use frontend::Frontend;

use crate::module::{
    ModuleFactory,
    ModuleSpecification,
};

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
    let mut monitor_manager = MonitorManager::new(host_manager.get_state_udpate_channel());
    let mut connection_manager = ConnectionManager::new();

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
            let monitor_spec = ModuleSpecification::new(monitor_id.clone(), monitor_config.version.as_str());
            let monitor = module_factory.new_monitor(&monitor_spec, &monitor_config.settings);

            // Initialize a connector if the monitors uses any.
            if let Some(connector_spec) = monitor.get_connector_spec() {
                let connector_settings = match host_config.connectors.get(&connector_spec.id) {
                    Some(config) => config.settings.clone(),
                    None => HashMap::new(),
                };

                let connector = module_factory.new_connector(&connector_spec, &connector_settings);

                let message_sender = connection_manager.add_connector(&host, connector);
                monitor_manager.add_monitor(&host, monitor, Some(message_sender));
            }
            else {
                monitor_manager.add_monitor(&host, monitor, None);
            }
        }
    }

    monitor_manager.refresh_monitors();
    let initial_display_data = host_manager.get_display_data();

    let mut frontend = frontend::qt::QmlFrontend::new(&initial_display_data);
    host_manager.add_observer(frontend.new_update_sender());

    frontend.start();

    connection_manager.join();
    monitor_manager.join();
    host_manager.join();

}