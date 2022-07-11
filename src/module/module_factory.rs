
use std::collections::HashMap;

use super::{
    Module,
    ModuleSpecification,
    connection,
    connection::ConnectionModule,
    monitoring,
    monitoring::MonitoringModule,
    command,
    command::CommandModule,
};


pub struct ModuleFactory {
    connector_constructors: HashMap<ModuleSpecification, fn(&HashMap<String, String>) -> connection::Connector>,
    monitor_constructors: HashMap<ModuleSpecification, fn(&HashMap<String, String>) -> monitoring::Monitor>,
    command_constructors: HashMap<ModuleSpecification, fn(&HashMap<String, String>) -> command::Command>,
}

impl ModuleFactory {
    pub fn new() -> Self {
        let mut manager = ModuleFactory {
            connector_constructors: HashMap::new(),
            monitor_constructors: HashMap::new(),
            command_constructors: HashMap::new(),
        };

        manager.load_modules();

        manager
    }

    pub fn new_connector(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> connection::Connector {
        match self.connector_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(settings),
            None => panic!("Required connection module '{}' not found", module_spec)
        }
    }

    pub fn new_monitor(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> monitoring::Monitor {
        match self.monitor_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(settings),
            None => panic!("Required monitoring module '{}' not found", module_spec)
        }
    }

    pub fn new_command(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> command::Command {
        match self.command_constructors.get(&module_spec)  {
            Some(constructor) => return constructor(settings),
            None => panic!("Required command module '{}' not found", module_spec)
        }
    }

    fn load_modules(&mut self) {
        log::info!("Loading modules");
        self.connector_constructors.insert(connection::Ssh2::get_metadata().module_spec, connection::Ssh2::new_connection_module);

        self.monitor_constructors.insert(monitoring::Uptime::get_metadata().module_spec, monitoring::Uptime::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::Ping::get_metadata().module_spec, monitoring::Ping::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::Ssh::get_metadata().module_spec, monitoring::Ssh::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::Docker::get_metadata().module_spec, monitoring::Docker::new_monitoring_module);

        self.command_constructors.insert(command::docker::Restart::get_metadata().module_spec, command::docker::Restart::new_command_module);
        self.command_constructors.insert(command::docker::Inspect::get_metadata().module_spec, command::docker::Inspect::new_command_module);
        self.command_constructors.insert(command::docker::Shell::get_metadata().module_spec, command::docker::Shell::new_command_module);
    }


}
