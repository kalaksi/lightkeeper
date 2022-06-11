
use std::collections::HashMap;
use super::{
    Module,
    ModuleSpecification,
    connection::ConnectionModule,
    connection::Connector,
    connection::ssh::Ssh2,
    monitoring::MonitoringModule,
    monitoring::Monitor,
    monitoring::linux::Uptime,
    monitoring::linux::Docker,
    monitoring::network::Ping,
    monitoring::network::Ssh,
};


pub struct ModuleFactory {
    connector_constructors: HashMap<ModuleSpecification, fn(&HashMap<String, String>) -> Connector>,
    monitor_constructors: HashMap<ModuleSpecification, fn(&HashMap<String, String>) -> Monitor>,
}

impl ModuleFactory {
    pub fn new() -> Self {
        let mut manager = ModuleFactory {
            connector_constructors: HashMap::new(),
            monitor_constructors: HashMap::new(),
        };

        manager.load_modules();

        manager
    }

    pub fn new_connector(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> Connector {
        match self.connector_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(settings),
            None => panic!("Required connection module '{}' not found", module_spec)
        }
    }

    pub fn new_monitor(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> Monitor {
        match self.monitor_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(settings),
            None => panic!("Required monitoring module '{}' not found", module_spec)
        }
    }

    fn load_modules(&mut self) {
        log::info!("Loading modules");
        self.connector_constructors.insert(Ssh2::get_metadata().module_spec, Ssh2::new_connection_module);
        self.monitor_constructors.insert(Uptime::get_metadata().module_spec, Uptime::new_monitoring_module);
        self.monitor_constructors.insert(Ping::get_metadata().module_spec, Ping::new_monitoring_module);
        self.monitor_constructors.insert(Ssh::get_metadata().module_spec, Ssh::new_monitoring_module);
        self.monitor_constructors.insert(Docker::get_metadata().module_spec, Docker::new_monitoring_module);
    }


}
