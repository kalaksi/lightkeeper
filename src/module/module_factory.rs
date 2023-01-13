
use std::collections::HashMap;

use super::{
    module::MetadataSupport,
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
        let normalized_spec = match module_spec.latest_version() {
            true => ModuleSpecification::new(&module_spec.id.as_str(), self.get_latest_version_for_connector(&module_spec.id).as_str()),
            false => module_spec.clone(),
        };

        let constructor = self.connector_constructors.get(&normalized_spec).unwrap();
        constructor(settings)
    }

    pub fn new_monitor(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> monitoring::Monitor {
        let normalized_spec = match module_spec.latest_version() {
            true => ModuleSpecification::new(&module_spec.id.as_str(), self.get_latest_version_for_monitor(&module_spec.id).as_str()),
            false => module_spec.clone(),
        };

        let constructor = self.monitor_constructors.get(&normalized_spec).unwrap();
        constructor(settings)
    }

    pub fn new_command(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> command::Command {
        let normalized_spec = match module_spec.latest_version() {
            true => ModuleSpecification::new(&module_spec.id.as_str(), self.get_latest_version_for_command(&module_spec.id).as_str()),
            false => module_spec.clone(),
        };

        let constructor = self.command_constructors.get(&normalized_spec).unwrap();
        constructor(settings)
    }

    pub fn get_latest_version_for_command(&self, module_id: &String) -> String {
        let mut all_versions = self.command_constructors.iter()
                                                        .filter(|(spec, _)| &spec.id == module_id)
                                                        .map(|(spec, _)| spec.version.clone())
                                                        .collect::<Vec<String>>();
        all_versions.sort();
        all_versions.last().unwrap_or_else(|| panic!("Command module '{}' was not found.", module_id)).to_owned()
    }

    pub fn get_latest_version_for_monitor(&self, module_id: &String) -> String {
        let mut all_versions = self.monitor_constructors.iter()
                                                        .filter(|(spec, _)| &spec.id == module_id)
                                                        .map(|(spec, _)| spec.version.clone())
                                                        .collect::<Vec<String>>();
        all_versions.sort();
        all_versions.last().unwrap_or_else(|| panic!("Monitoring module '{}' was not found.", module_id)).to_owned()
    }

    pub fn get_latest_version_for_connector(&self, module_id: &String) -> String {
        let mut all_versions = self.connector_constructors.iter()
                                                          .filter(|(spec, _)| &spec.id == module_id)
                                                          .map(|(spec, _)| spec.version.clone())
                                                          .collect::<Vec<String>>();
        all_versions.sort();
        all_versions.last().unwrap_or_else(|| panic!("Connector module '{}' was not found.", module_id)).to_owned()
    }

    fn load_modules(&mut self) {
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
        self.command_constructors.insert(command::docker::compose::Stop::get_metadata().module_spec, command::docker::compose::Stop::new_command_module);
        self.command_constructors.insert(command::systemd::service::Start::get_metadata().module_spec, command::systemd::service::Start::new_command_module);
        self.command_constructors.insert(command::systemd::service::Stop::get_metadata().module_spec, command::systemd::service::Stop::new_command_module);

        log::info!("Loaded {} command modules, {} monitoring modules and {} connector modules",
                   self.command_constructors.len(), self.monitor_constructors.len(), self.connector_constructors.len());

    }



}
