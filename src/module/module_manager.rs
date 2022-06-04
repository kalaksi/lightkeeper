
use std::collections::HashMap;
use super::{
    Module,
    ModuleSpecification,
    connection::ConnectionModule,
    connection::Empty,
    connection::ssh::Ssh2,
    monitoring::MonitoringModule,
    monitoring::linux::Uptime,
    monitoring::linux::Docker,
    monitoring::network::Ping,
    monitoring::network::Ssh,
};


pub struct ModuleManager {
    connection_constructors: HashMap<ModuleSpecification, fn(&HashMap<String, String>) -> Box<dyn ConnectionModule>>,
    monitoring_constructors: HashMap<ModuleSpecification, fn(&HashMap<String, String>) -> Box<dyn MonitoringModule>>,
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

    pub fn new_connection_module(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> Box<dyn ConnectionModule> {
        match self.connection_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(settings),
            None => panic!("Required connection module '{}' not found", module_spec)
        }
    }

    pub fn new_monitoring_module(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> Box<dyn MonitoringModule> {
        match self.monitoring_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(settings),
            None => panic!("Required monitoring module '{}' not found", module_spec)
        }
    }

    fn load_modules(&mut self) {
        log::info!("Loading modules");
        self.connection_constructors.insert(Empty::get_metadata().module_spec, Empty::new_connection_module);
        self.connection_constructors.insert(Ssh2::get_metadata().module_spec, Ssh2::new_connection_module);
        self.monitoring_constructors.insert(Uptime::get_metadata().module_spec, Uptime::new_monitoring_module);
        self.monitoring_constructors.insert(Ping::get_metadata().module_spec, Ping::new_monitoring_module);
        self.monitoring_constructors.insert(Ssh::get_metadata().module_spec, Ssh::new_monitoring_module);
        self.monitoring_constructors.insert(Docker::get_metadata().module_spec, Docker::new_monitoring_module);
    }


}
