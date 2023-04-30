
use std::collections::HashMap;

use super::{
    module::MetadataSupport,
    Metadata,
    ModuleSpecification,
    connection,
    connection::ConnectionModule,
    monitoring,
    monitoring::MonitoringModule,
    command,
    command::CommandModule,
};


pub struct ModuleFactory {
    connector_modules: Vec<(Metadata, fn(&HashMap<String, String>) -> connection::Connector)>,
    monitor_modules: Vec<(Metadata, fn(&HashMap<String, String>) -> monitoring::Monitor)>,
    command_modules: Vec<(Metadata, fn(&HashMap<String, String>) -> command::Command)>,
}

impl ModuleFactory {
    pub fn new() -> Self {
        let mut manager = ModuleFactory {
            connector_modules: Vec::new(),
            monitor_modules: Vec::new(),
            command_modules: Vec::new(),
        };

        manager.load_modules();

        manager
    }

    pub fn new_connector(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> connection::Connector {
        let normalized_spec = match module_spec.latest_version() {
            true => ModuleSpecification::new(&module_spec.id.as_str(), self.get_latest_version_for_connector(&module_spec.id).as_str()),
            false => module_spec.clone(),
        };

        let constructor = self.connector_modules.iter().find(|(metadata, _ctor)| metadata.module_spec == normalized_spec).unwrap().1;
        constructor(settings)
    }

    pub fn new_monitor(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> monitoring::Monitor {
        let normalized_spec = match module_spec.latest_version() {
            true => ModuleSpecification::new(&module_spec.id.as_str(), self.get_latest_version_for_monitor(&module_spec.id).as_str()),
            false => module_spec.clone(),
        };

        let constructor = self.monitor_modules.iter().find(|(metadata, _ctor)| metadata.module_spec == normalized_spec).unwrap().1;
        constructor(settings)
    }

    pub fn new_command(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> command::Command {
        let normalized_spec = match module_spec.latest_version() {
            true => ModuleSpecification::new(&module_spec.id.as_str(), self.get_latest_version_for_command(&module_spec.id).as_str()),
            false => module_spec.clone(),
        };

        let constructor = self.command_modules.iter().find(|(metadata, _ctor)| metadata.module_spec == normalized_spec).unwrap().1;
        constructor(settings)
    }

    pub fn get_latest_version_for_command(&self, module_id: &String) -> String {
        let mut all_versions = self.command_modules.iter()
                                                   .filter(|(metadata, _)| &metadata.module_spec.id == module_id)
                                                   .map(|(metadata, _)| metadata.module_spec.version.clone())
                                                   .collect::<Vec<String>>();
        all_versions.sort();
        all_versions.last().unwrap_or_else(|| panic!("Command module '{}' was not found.", module_id)).to_owned()
    }

    pub fn get_latest_version_for_monitor(&self, module_id: &String) -> String {
        let mut all_versions = self.monitor_modules.iter()
                                                   .filter(|(metadata, _)| &metadata.module_spec.id == module_id)
                                                   .map(|(metadata, _)| metadata.module_spec.version.clone())
                                                   .collect::<Vec<String>>();
        all_versions.sort();
        all_versions.last().unwrap_or_else(|| panic!("Monitoring module '{}' was not found.", module_id)).to_owned()
    }

    pub fn get_latest_version_for_connector(&self, module_id: &String) -> String {
        let mut all_versions = self.connector_modules.iter()
                                                     .filter(|(metadata, _)| &metadata.module_spec.id == module_id)
                                                     .map(|(metadata, _)| metadata.module_spec.version.clone())
                                                     .collect::<Vec<String>>();
        all_versions.sort();
        all_versions.last().unwrap_or_else(|| panic!("Connector module '{}' was not found.", module_id)).to_owned()
    }

    pub fn validate_modules(&self) {
        log::info!("Validating module configuration");

        // Validate monitoring modules.
        for (metadata, constructor) in self.monitor_modules.iter() {
            let new_monitor = constructor(&HashMap::new());
            if let Err(error) = new_monitor.get_display_options().validate() {
                panic!("Error in monitoring module '{}' display_options: {}", metadata.module_spec.id, error);
            }

            if let Some(connector_spec) = new_monitor.get_connector_spec() {
                self.connector_modules.iter()
                    .find(|(metadata, _ctor)| metadata.module_spec == connector_spec)
                    .unwrap_or_else(|| panic!("Connector module '{}' for monitoring module '{}' was not found.",
                        connector_spec.id, metadata.module_spec.id));
            }

            if let Some(parent_spec) = &metadata.parent_module {
                let matches = self.monitor_modules.iter().filter(|(metadata, _)| &metadata.module_spec == parent_spec).collect::<Vec<_>>();
                if matches.len() == 0 {
                    panic!("Parent module '{}' for monitoring extension module '{}' was not found.", parent_spec.id, metadata.module_spec.id);
                }
                else if matches.len() > 1 {
                    let extension_modules = matches.iter().map(|(metadata, _)| metadata.module_spec.clone().id).collect::<Vec<_>>().join(", ");
                    panic!("Multiple extension modules for monitoring module '{}' were found ({})", parent_spec.id, extension_modules);
                }
            }
        }

        // Validate command modules.
        for (metadata, constructor) in self.command_modules.iter() {
            let new_command = constructor(&HashMap::new());
            if let Err(error) = new_command.get_display_options().validate() {
                panic!("Error in command module '{}' display_options: {}", metadata.module_spec.id, error);
            }

            if let Some(connector_spec) = new_command.get_connector_spec() {
                self.connector_modules.iter()
                    .find(|(metadata, _ctor)| metadata.module_spec == connector_spec)
                    .unwrap_or_else(|| panic!("Connector module '{}' for command module '{}' was not found.",
                        connector_spec.id, metadata.module_spec.id));
            }

        }
    }

    fn load_modules(&mut self) {
        // Connection modules.
        self.connector_modules = vec![
            (connection::Ssh2::get_metadata(), connection::Ssh2::new_connection_module),
        ];

        // Monitoring modules.
        self.monitor_modules = vec![
            (monitoring::os::Os::get_metadata(), monitoring::os::Os::new_monitoring_module),
            (monitoring::linux::Package::get_metadata(), monitoring::linux::Package::new_monitoring_module),
            (monitoring::linux::systemd::Service::get_metadata(), monitoring::linux::systemd::Service::new_monitoring_module),
            (monitoring::linux::Kernel::get_metadata(), monitoring::linux::Kernel::new_monitoring_module),
            (monitoring::linux::Filesystem::get_metadata(), monitoring::linux::Filesystem::new_monitoring_module),
            (monitoring::linux::Interface::get_metadata(), monitoring::linux::Interface::new_monitoring_module),
            (monitoring::linux::Uptime::get_metadata(), monitoring::linux::Uptime::new_monitoring_module),
            (monitoring::network::Ping::get_metadata(), monitoring::network::Ping::new_monitoring_module),
            (monitoring::network::Ssh::get_metadata(), monitoring::network::Ssh::new_monitoring_module),
            (monitoring::docker::Compose::get_metadata(), monitoring::docker::Compose::new_monitoring_module),
            (monitoring::docker::Containers::get_metadata(), monitoring::docker::Containers::new_monitoring_module),
            (monitoring::docker::Images::get_metadata(), monitoring::docker::Images::new_monitoring_module),

            // Monitoring extension modules.
            (monitoring::docker::ImageUpdates::get_metadata(), monitoring::docker::ImageUpdates::new_monitoring_module),
        ];

        // Command modules.
        self.command_modules = vec![
            (command::linux::Logs::get_metadata(), command::linux::Logs::new_command_module),
            (command::linux::Reboot::get_metadata(), command::linux::Reboot::new_command_module),
            (command::linux::Shutdown::get_metadata(), command::linux::Shutdown::new_command_module),
            (command::linux::packages::Clean::get_metadata(), command::linux::packages::Clean::new_command_module),
            (command::linux::packages::Update::get_metadata(), command::linux::packages::Update::new_command_module),
            (command::linux::packages::UpdateAll::get_metadata(), command::linux::packages::UpdateAll::new_command_module),
            (command::docker::Restart::get_metadata(), command::docker::Restart::new_command_module),
            (command::docker::Inspect::get_metadata(), command::docker::Inspect::new_command_module),
            (command::docker::Shell::get_metadata(), command::docker::Shell::new_command_module),
            (command::docker::image::Remove::get_metadata(), command::docker::image::Remove::new_command_module),
            (command::docker::image::Prune::get_metadata(), command::docker::image::Prune::new_command_module),
            (command::docker::compose::Edit::get_metadata(), command::docker::compose::Edit::new_command_module),
            (command::docker::compose::Pull::get_metadata(), command::docker::compose::Pull::new_command_module),
            (command::docker::compose::Up::get_metadata(), command::docker::compose::Up::new_command_module),
            (command::docker::compose::Start::get_metadata(), command::docker::compose::Start::new_command_module),
            (command::docker::compose::Stop::get_metadata(), command::docker::compose::Stop::new_command_module),
            (command::systemd::service::Start::get_metadata(), command::systemd::service::Start::new_command_module),
            (command::systemd::service::Stop::get_metadata(), command::systemd::service::Stop::new_command_module),
            (command::systemd::service::Mask::get_metadata(), command::systemd::service::Mask::new_command_module),
            (command::systemd::service::Unmask::get_metadata(), command::systemd::service::Unmask::new_command_module),
        ];

        self.validate_modules();
        log::info!("Loaded {} command modules, {} monitoring modules and {} connector modules",
                   self.command_modules.len(), self.monitor_modules.len(), self.connector_modules.len());

    }

}
