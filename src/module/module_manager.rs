use std::collections::HashMap;

use super::{
    Module,
    ModuleSpecification,
    connection::ConnectionModule,
    connection::ssh::Ssh2,
    monitoring::MonitoringModule,
    monitoring::linux::Uptime,
};


pub struct ModuleManager {
    connection_constructors: HashMap<ModuleSpecification, fn() -> Box<dyn ConnectionModule>>,
    monitoring_constructors: HashMap<ModuleSpecification, fn() -> Box<dyn MonitoringModule>>,
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

    pub fn new_connection_module(&self, module_spec: &ModuleSpecification) -> Box<dyn ConnectionModule> {
        match self.connection_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(),
            None => panic!("Required connection module '{}' not found", module_spec)
        }
    }

    pub fn new_monitoring_module(&self, module_spec: &ModuleSpecification) -> Box<dyn MonitoringModule> {
        match self.monitoring_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(),
            None => panic!("Required monitoring module '{}' not found", module_spec)
        }
    }

    fn load_modules(&mut self) {
        log::info!("Loading modules");
        self.connection_constructors.insert(Ssh2::get_metadata().module_spec, Ssh2::new_connection_module);
        self.monitoring_constructors.insert(Uptime::get_metadata().module_spec, Uptime::new_monitoring_module);
    }


}
