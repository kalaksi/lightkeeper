mod module;
mod host_manager;
mod host;

use clap::Parser;
use host_manager::HostManager;
use host::Host;

use crate::module::{
    ModuleManager,
    monitoring::linux::Uptime,
    monitoring::MonitoringModule,
    connection::AuthenticationDetails,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    level: String,
    command: String,
}


fn main() {
    env_logger::init();
    // let args = Args::parse();

    log::info!("Lightkeeper starting...");
    let module_manager = ModuleManager::new();

    let mut host_manager = HostManager::new(&module_manager);

    let mut host = Host::new(&String::from("test"));
    host.set_address("10.4.0.2", 22);


    host_manager.add_host(host);


    let authentication = AuthenticationDetails::new(Some(String::from("test")), None);

    let connector = match host_manager.get_connector(&String::from("test"), &String::from("ssh"), Some(authentication)) {
        Ok(connector) => connector,
        Err(error) => { log::error!("Error while connecting: {}", error); return }
    };


    let monitor = Uptime::new_monitoring_module();
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