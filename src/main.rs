mod module;
mod host_manager;
mod host;
mod configuration;
mod utils;

use std::collections::HashMap;
use clap::Parser;
use tabled::{ Table, Style };

use host_manager::HostManager;
use host::Host;
use configuration::Configuration;

use crate::module::{
    ModuleManager,
    ModuleSpecification,
    monitoring::MonitoringModule,
    connection::AuthenticationDetails,
};

#[derive(Parser)]
#[clap()]
struct Args {
    #[clap(short, long, default_value = "config.toml")]
    config_file: String,
    host: String,
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

    let module_manager = ModuleManager::new();
    let mut host_manager = HostManager::new(&module_manager);

    let mut host_monitors: HashMap<String, Vec<Box<dyn MonitoringModule>>> = HashMap::new();

    for host_details in &config.hosts {
        log::info!("Found configuration for host {}", host_details.name);

        let mut host = Host::new(&host_details.name);
        host.set_address(&host_details.address);
        match host_manager.add_host(host) {
            Ok(()) => (),
            Err(error) => {
                log::error!("{}", error.to_string());
                continue;
            }
        };

        host_monitors.insert(host_details.name.clone(), Vec::new());

        for monitor in &host_details.monitors {
            let module_spec = ModuleSpecification::from_string(&monitor).unwrap();
            host_monitors.get_mut(&host_details.name).unwrap().push(module_manager.new_monitoring_module(&module_spec));
        }
    }

    for host_config in &config.hosts {
        let monitors = host_monitors.get(&host_config.name).unwrap();
        let host = host_manager.get_host(&host_config.name).unwrap();
        
        for monitor in monitors {
            let authentication = AuthenticationDetails::new(&config.authentication.username, &config.authentication.password);
            let connector = match host_manager.get_connector(&host.name, &monitor.get_connector_spec(), Some(authentication)) {
                Ok(connector) => connector,
                Err(error) => {
                    log::error!("Error while connecting: {}", error);
                    continue;
                }
            };

            match monitor.refresh(&host, connector) {
                Ok(data) => {
                    host_manager.insert_monitoring_data(&host.name, &monitor.get_module_spec().id, data)
                                .expect("Failed to store monitoring data");
                }
                Err(error) => {
                    log::error!("Error while refreshing monitoring data: {}", error);
                }
            };
        }
    }

}