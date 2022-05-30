mod module;
mod host_manager;
mod host;
mod configuration;
mod utils;
mod frontend;

use std::collections::HashMap;
use clap::Parser;

use host_manager::HostManager;
use host::Host;
use configuration::Configuration;
use frontend::Frontend;

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

    for host_config in &config.hosts {
        log::info!("Found configuration for host {}", host_config.name);

        let mut host = Host::new(&host_config.name);
        if let Err(error) = host.set_address(&host_config.address) {
            log::error!("{}", error.to_string());
            continue;
        }

        if let Err(error) = host_manager.add_host(host) {
            log::error!("{}", error.to_string());
            continue;
        };

        host_monitors.insert(host_config.name.clone(), Vec::new());

        for monitor in &host_config.monitors {
            let module_spec = ModuleSpecification::from_string(&monitor).unwrap();
            host_monitors.get_mut(&host_config.name).unwrap().push(module_manager.new_monitoring_module(&module_spec));
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

    frontend::cli::Cli::draw(&host_manager.get_display_data());

}