use std::{ collections::HashMap };

use super::{
    Module,
    connection::ConnectionModule,
    connection::ssh::Ssh2,
    monitoring::MonitoringModule,
    monitoring::linux::Uptime,
};


pub struct ModuleManager {
    connection_constructors: HashMap<String, fn() -> Box<dyn ConnectionModule>>,
    monitoring_constructors: HashMap<String, fn() -> Box<dyn MonitoringModule>>,
}

impl ModuleManager {
    pub fn new() -> Self {
        let mut manager = ModuleManager {
            connection_constructors: HashMap::new(),
            monitoring_constructors: HashMap::new(),
        };

        manager.load_modules();

        manager
    }

    pub fn new_connection_module(&self, name: &String) -> Box<dyn ConnectionModule> {
        self.connection_constructors.get(name).unwrap()()
    }

    fn load_modules(&mut self) {
        log::info!("Loading modules");
        self.connection_constructors.insert(Ssh2::get_metadata().name, Ssh2::new_connection_module);
        self.monitoring_constructors.insert(Uptime::get_metadata().name, Uptime::new_monitoring_module);
    }


}
