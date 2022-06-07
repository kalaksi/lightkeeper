mod module;
mod host_manager;
mod host;
mod configuration;
mod utils;
mod frontend;

use std::{ collections::HashMap };

use clap::Parser;

use host_manager::HostManager;
use host::Host;
use configuration::Configuration;
use frontend::Frontend;

use crate::module::{
    ModuleFactory,
    ModuleSpecification,
    monitoring::MonitoringModule,
    monitoring::DataPoint,
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
    let mut host_manager = HostManager::new(&module_factory);

    // Host name as first key, monitor name as second.
    let mut host_monitors: HashMap<String, HashMap<String, Box<dyn MonitoringModule>>> = HashMap::new();

    // Configure hosts and modules.
    for (host_name, host_config) in config.hosts.iter() {
        log::info!("Found configuration for host {}", host_name);
        let host = match Host::new(&host_name, &host_config.address, &host_config.fqdn) {
            Ok(host) => host,
            Err(error) => {
                log::error!("{}", error);
                continue;
            }
        };

        host_monitors.insert(host_name.clone(), HashMap::new());
        let mut critical_monitors: Vec<String> = Vec::new();

        for (monitor_name, monitor) in host_config.monitors.iter() {
            let module_spec = ModuleSpecification::new(monitor_name.clone(), monitor.version.clone());
            let module_instance = module_factory.new_monitoring_module(&module_spec, &monitor.settings);

            host_monitors.get_mut(host_name).unwrap().insert(monitor_name.clone(), module_instance);

            if monitor.is_critical.unwrap_or(false) {
                log::debug!("Adding critical monitor {}", monitor_name);
                critical_monitors.push(monitor_name.clone());
            }
        }

        if let Err(error) = host_manager.add_host(host, critical_monitors, config.general.default_host_status) {
            log::error!("{}", error.to_string());
            continue;
        };
    }

    // Refresh monitoring data.
    for (host_name, host_config) in config.hosts.iter() {
        let monitors = host_monitors.get_mut(host_name).unwrap();
        let host = host_manager.get_host(host_name).unwrap();

        log::info!("Refreshing monitoring data for host {}", host.name);
        
        for (monitor_name, monitor) in monitors.iter_mut() {
            let monitor_settings = &host_config.monitors.get(monitor_name).unwrap().settings;

            let new_data_result = match host_manager.get_connector(&host.name, &monitor.get_connector_spec(), monitor_settings) {
                Ok(connector) => monitor.refresh(&host, connector),
                Err(error) => Err(format!("Error while connecting: {}", error))
            };

            let new_data = new_data_result.unwrap_or_else(|error| {
                log::info!("Error while refreshing monitoring data: {}: {}", monitor_name, error);
                DataPoint::empty_and_critical()
            });

            host_manager.insert_monitoring_data(&host.name, monitor, new_data)
                        .expect("Failed to store monitoring data");

        }
    }

    frontend::cli::Cli::draw(&host_manager.get_display_data(&config.display_options.excluded_monitors));

}