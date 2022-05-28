mod module;
mod host_manager;
mod host;
mod configuration;

use clap::Parser;
use host_manager::HostManager;
use host::Host;

use crate::{module::{
    ModuleManager,
    monitoring::linux::Uptime,
    monitoring::MonitoringModule,
    connection::AuthenticationDetails,
}, configuration::Configuration};

#[derive(Parser)]
#[clap()]
struct Args {
    #[clap(short, long, default_value = "config.toml")]
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

    let authentication = AuthenticationDetails::new(&config.username, &config.password);
    let module_manager = ModuleManager::new();
    let mut host_manager = HostManager::new(&module_manager);

    let mut host = Host::new(&String::from("test"));
    host.set_address("10.4.0.2", 22);
    host_manager.add_host(host);


    let connector = match host_manager.get_connector(&String::from("test"), &String::from("ssh"), Some(authentication)) {
        Ok(connector) => connector,
        Err(error) => { log::error!("Error while connecting: {}", error); return }
    };

    let monitor = Uptime::new_monitoring_module();
    let connector_spec = monitor.get_connector_spec();

    if !connector_spec.is_acceptable(connector) {
        log::error!("Connector module not found or version incompatible ({})", connector_spec);
        return;
    }

    match monitor.refresh(connector) {
        Ok(data) => {
            log::info!("Got {}", data.value);
        }
        Err(error) => {
            log::error!("Error while refreshing monitoring data: {}", error);
            Default::default()
        }
    };

    
}