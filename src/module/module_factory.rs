
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

        self.monitor_constructors.insert(monitoring::linux::Package::get_metadata().module_spec, monitoring::linux::Package::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::linux::systemd::Service::get_metadata().module_spec, monitoring::linux::systemd::Service::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::linux::Kernel::get_metadata().module_spec, monitoring::linux::Kernel::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::linux::Filesystem::get_metadata().module_spec, monitoring::linux::Filesystem::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::linux::Interface::get_metadata().module_spec, monitoring::linux::Interface::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::linux::Uptime::get_metadata().module_spec, monitoring::linux::Uptime::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::network::Ping::get_metadata().module_spec, monitoring::network::Ping::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::network::Ssh::get_metadata().module_spec, monitoring::network::Ssh::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::docker::Compose::get_metadata().module_spec, monitoring::docker::Compose::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::docker::Containers::get_metadata().module_spec, monitoring::docker::Containers::new_monitoring_module);
        self.monitor_constructors.insert(monitoring::docker::Images::get_metadata().module_spec, monitoring::docker::Images::new_monitoring_module);

        self.command_constructors.insert(command::linux::Logs::get_metadata().module_spec, command::linux::Logs::new_command_module);
        self.command_constructors.insert(command::linux::Reboot::get_metadata().module_spec, command::linux::Reboot::new_command_module);
        self.command_constructors.insert(command::linux::Shutdown::get_metadata().module_spec, command::linux::Shutdown::new_command_module);
        self.command_constructors.insert(command::docker::Restart::get_metadata().module_spec, command::docker::Restart::new_command_module);
        self.command_constructors.insert(command::docker::Inspect::get_metadata().module_spec, command::docker::Inspect::new_command_module);
        self.command_constructors.insert(command::docker::Shell::get_metadata().module_spec, command::docker::Shell::new_command_module);
        self.command_constructors.insert(command::docker::image::Remove::get_metadata().module_spec, command::docker::image::Remove::new_command_module);
        self.command_constructors.insert(command::docker::image::Prune::get_metadata().module_spec, command::docker::image::Prune::new_command_module);
        self.command_constructors.insert(command::docker::compose::Edit::get_metadata().module_spec, command::docker::compose::Edit::new_command_module);
        self.command_constructors.insert(command::docker::compose::Pull::get_metadata().module_spec, command::docker::compose::Pull::new_command_module);
        self.command_constructors.insert(command::docker::compose::Up::get_metadata().module_spec, command::docker::compose::Up::new_command_module);
        self.command_constructors.insert(command::docker::compose::Start::get_metadata().module_spec, command::docker::compose::Start::new_command_module);
    }


}
