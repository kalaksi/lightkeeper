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
    let mut monitor_manager = MonitorManager::new();
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
            let module_spec = ModuleSpecification::new(monitor_id.clone(), monitor_config.version.clone());
            let monitor = module_factory.new_monitor(&module_spec, &monitor_config.settings);

            // TODO: connector settings
            let connector = module_factory.new_connector(&monitor.get_connector_spec(), &HashMap::new());
            let message_sender = connection_manager.add_connector(&host, connector);

            monitor_manager.add_monitor(&host, monitor, message_sender);
        }
    }


    frontend::cli::Cli::draw(&host_manager.get_display_data(&config.display_options.excluded_monitors));

    monitor_manager.join();
    connection_manager.join();

}