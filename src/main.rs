mod module;
mod host_manager;
mod host;
mod configuration;
mod utils;
mod frontend;

use std::{
    collections::HashMap,
    net::{self, ToSocketAddrs},
    str::FromStr,
};

use clap::Parser;

use host_manager::HostManager;
use host::Host;
use configuration::Configuration;
use frontend::Frontend;

use crate::module::{
    ModuleManager,
    ModuleSpecification,
    monitoring::{MonitoringModule, MonitoringData},
    connection::Credentials,
};

#[derive(Parser)]
#[clap()]
struct Args {
    #[clap(short, long, default_value = "config.yml")]
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

    // Configure hosts and modules.
    for host_config in &config.hosts {
        log::info!("Found configuration for host {}", host_config.name);
        let mut host = Host::new(&host_config.name);

        // 0.0.0.0 is the value for "unspecified".
        let ip_address_str = host_config.address.clone().unwrap_or(String::from("0.0.0.0"));
        host.ip_address = match net::Ipv4Addr::from_str(&ip_address_str) {
            Ok(address) => net::IpAddr::V4(address),
            Err(error) => {
                log::error!("{}", error.to_string());
                continue;
            }
        };

        host.fqdn = host_config.fqdn.clone().unwrap_or(String::from(""));

        if host.ip_address.is_unspecified() {
            if host.fqdn == "" {
                log::error!("Host {} does not have FQDN or IP address defined.", host.name);
                continue;
            }
            else {
                // Resolve FQDN and get the first IP address. Panic if nothing found.
                host.ip_address = format!("{}:0", host.fqdn).to_socket_addrs().unwrap().next().unwrap().ip();
            }
        }

        host_monitors.insert(host_config.name.clone(), Vec::new());
        let mut critical_monitors: Vec<String> = Vec::new();

        for monitor in &host_config.monitors {
            let module_spec = ModuleSpecification::new(monitor.name.clone(), monitor.version.clone());
            host_monitors.get_mut(&host_config.name).unwrap().push(module_manager.new_monitoring_module(&module_spec));

            if monitor.is_critical.unwrap_or(false) {
                log::debug!("Adding critical monitor {}", monitor.name);
                critical_monitors.push(monitor.name.clone());
            }
        }

        if let Err(error) = host_manager.add_host(host, critical_monitors, config.general.default_host_status) {
            log::error!("{}", error.to_string());
            continue;
        };
    }

    // Refresh monitoring data.
    for host_config in &config.hosts {
        let monitors = host_monitors.get_mut(&host_config.name).unwrap();
        let host = host_manager.get_host(&host_config.name).unwrap();

        log::info!("Refreshing monitoring data for host {}", host.name);
        
        for monitor in monitors {
            let credentials = Credentials::new(&config.authentication.username, &config.authentication.password);

            let new_data_result = match host_manager.get_connector(&host.name, &monitor.get_connector_spec(), Some(credentials)) {
                Ok(connector) => monitor.refresh(&host, connector),
                Err(error) => Err(format!("Error while connecting: {}", error))
            };

            let new_data = new_data_result.unwrap_or_else(|error| {
                log::info!("Error while refreshing monitoring data: {}", error);
                MonitoringData::empty_and_critical()
            });

            host_manager.insert_monitoring_data(&host.name, &monitor.get_module_spec().id, new_data)
                        .expect("Failed to store monitoring data");

        }
    }

    frontend::cli::Cli::draw(&host_manager.get_display_data(&config.display_options.excluded_monitors));

}