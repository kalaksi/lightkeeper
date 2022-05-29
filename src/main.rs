mod module;
mod host_manager;
mod host;
mod configuration;
mod utils;

use std::collections::HashMap;

use clap::Parser;
use host_manager::HostManager;
use host::Host;

use crate::{module::{
    ModuleManager,
    monitoring::MonitoringModule,
    connection::AuthenticationDetails, ModuleSpecification,
}, configuration::Configuration};

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
        host_manager.add_host(host);

        host_monitors.insert(host_details.name.clone(), Vec::new());

        for monitor in &host_details.monitors {
            let module_spec = ModuleSpecification::from_string(&monitor).unwrap();
            host_monitors.get_mut(&host_details.name).unwrap().push(module_manager.new_monitoring_module(&module_spec));
        }
    }

    for host in &config.hosts {
        let monitors = host_monitors.get(&host.name).unwrap();
        let host = host_manager.get_host(&host.name).unwrap();
        
        for monitor in monitors {
            let authentication = AuthenticationDetails::new(&config.authentication.username, &config.authentication.password);
            let connector = match host_manager.get_connector(&host.name, &monitor.get_connector_spec(), Some(authentication)) {
                Ok(connector) => connector,
                Err(error) => { log::error!("Error while connecting: {}", error); return }
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