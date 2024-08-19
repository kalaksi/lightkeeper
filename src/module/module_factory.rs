
use std::collections::HashMap;

use super::{
    module::MetadataSupport,
    Metadata,
    ModuleSpecification,
    ModuleType,
    connection,
    connection::ConnectionModule,
    monitoring,
    monitoring::MonitoringModule,
    command,
    command::CommandModule,
};


#[derive(Default)]
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
        let mut normalized_spec = module_spec.clone();
        if normalized_spec.latest_version() {
            normalized_spec.version = self.get_latest_version_for_connector(&normalized_spec.id);
        }

        let constructor = self.connector_modules.iter().find(|(metadata, _ctor)| metadata.module_spec == normalized_spec).unwrap().1;
        constructor(settings)
    }

    pub fn new_monitor(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> monitoring::Monitor {
        let mut normalized_spec = module_spec.clone();
        if normalized_spec.latest_version() {
            normalized_spec.version = self.get_latest_version_for_monitor(&normalized_spec.id);
        }

        let constructor = self.monitor_modules.iter().find(|(metadata, _ctor)| metadata.module_spec == normalized_spec).unwrap().1;
        constructor(settings)
    }

    pub fn new_command(&self, module_spec: &ModuleSpecification, settings: &HashMap<String, String>) -> Option<command::Command> {
        let mut normalized_spec = module_spec.clone();
        if normalized_spec.latest_version() {
            if let Some(latest_version) = self.get_latest_version_for_command(&normalized_spec.id) {
                normalized_spec.version = latest_version
            }
            else {
                log::error!("Command module '{}' was not found.", normalized_spec.id);
                return None;
            }
        }

        let constructor = self.command_modules.iter().find(|(metadata, _ctor)| metadata.module_spec == normalized_spec).unwrap().1;
        Some(constructor(settings))
    }

    pub fn get_latest_version_for_command(&self, module_id: &String) -> Option<String> {
        let mut all_versions = self.command_modules.iter()
                                                   .filter(|(metadata, _)| &metadata.module_spec.id == module_id)
                                                   .map(|(metadata, _)| metadata.module_spec.version.clone())
                                                   .collect::<Vec<String>>();
        all_versions.sort();
        all_versions.last().cloned()
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

    pub fn get_connector_module_metadata(&self, module_spec: &ModuleSpecification) -> Metadata {
        let module_spec = ModuleSpecification::new(module_spec.id.as_str(), module_spec.version.as_str(), ModuleType::Connector);
        self.connector_modules.iter().find(|(metadata, _ctor)| metadata.module_spec == module_spec).unwrap().0.clone()
    }

    pub fn get_module_metadatas(&self) -> Vec<Metadata> {
        let mut metadatas = Vec::new();
        metadatas.extend(self.connector_modules.iter().map(|(metadata, _ctor)| metadata.clone()));
        metadatas.extend(self.monitor_modules.iter().map(|(metadata, _ctor)| metadata.clone()));
        metadatas.extend(self.command_modules.iter().map(|(metadata, _ctor)| metadata.clone()));
        metadatas
    }

    pub fn validate_modules(&self) {
        log::info!("Validating modules");

        // Validate monitoring modules.
        for (metadata, constructor) in self.monitor_modules.iter() {
            let new_monitor = constructor(&HashMap::new());
            if let Err(error) = new_monitor.get_display_options().validate() {
                panic!("Error in monitoring module '{}' display_options: {}", metadata.module_spec.id, error);
            }

            if let Some(connector_spec) = new_monitor.get_connector_spec() {
                if connector_spec.module_type != ModuleType::Connector {
                    panic!("Invalid connector module type for monitoring module '{}'.", metadata.module_spec.id);
                }

                self.connector_modules.iter()
                    .find(|(metadata, _ctor)| metadata.module_spec == connector_spec)
                    .unwrap_or_else(|| panic!("Connector module '{}' for monitoring module '{}' was not found.",
                        connector_spec.id, metadata.module_spec.id));
            }

            if let Some(parent_spec) = &metadata.parent_module {
                if parent_spec.module_type != ModuleType::Monitor {
                    panic!("Invalid parent module type for monitoring module '{}'.", metadata.module_spec.id);
                }

                let matches = self.monitor_modules.iter().filter(|(metadata, _)| metadata.module_spec == *parent_spec).collect::<Vec<_>>();
                if matches.is_empty() {
                    panic!("Parent module '{}' for monitoring extension module '{}' was not found.", parent_spec.id, metadata.module_spec.id);
                }
                // Currently, multiple extension modules for the same parent/base are not supported.
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
                let connector_spec = ModuleSpecification::new_with_type(connector_spec.id.as_str(), connector_spec.version.as_str(), ModuleType::Connector);
                self.connector_modules.iter()
                    .find(|(metadata, _ctor)| metadata.module_spec == connector_spec)
                    .unwrap_or_else(|| panic!("Connector module '{}' for command module '{}' was not found.",
                        connector_spec.id, metadata.module_spec.id));
            }
        }
    }

    pub fn get_monitoring_module_info(&self) -> String {
        let mut documentation = String::from("Monitoring modules:\n");

        for (metadata, module_constructor) in self.monitor_modules.iter() {
            let module_instance = module_constructor(&HashMap::new());

            documentation.push_str(&format!("{}:\n", metadata.module_spec));

            if let Some(parent_spec) = &metadata.parent_module {
                documentation.push_str(&format!("  extends module: {}\n", parent_spec));
            };

            documentation.push_str(&format!("  description: {}\n", &metadata.description.replace("    ", "  ")));
            documentation.push_str("  settings:\n");
            if metadata.settings.is_empty() {
                for (key, value) in metadata.settings.iter() {
                    documentation.push_str(&format!("    {}: {}\n", key, value));
                }
            }
            else {
                documentation.push_str("    none\n");
            }

            match module_instance.get_connector_spec() {
                Some(connector_spec) => documentation.push_str(&format!("  connector: {}\n", connector_spec.id)),
                None => documentation.push_str("  connector: none\n"),
            };

            documentation.push('\n');
        }
        documentation
    }

    pub fn get_command_module_info(&self) -> String {
        let mut documentation = String::from("Command modules:\n");

        for (metadata, module_constructor) in self.command_modules.iter() {
            let module_instance = module_constructor(&HashMap::new());

            documentation.push_str(&format!("{}:\n", metadata.module_spec));

            if let Some(parent_spec) = &metadata.parent_module {
                documentation.push_str(&format!("extends module: {}\n", parent_spec));
            };

            documentation.push_str(&format!("  description: {}\n", &metadata.description.replace("    ", "  ")));
            documentation.push_str("  settings:\n");
            if metadata.settings.len() > 0 {
                for (key, value) in metadata.settings.iter() {
                    documentation.push_str(&format!("    {}: {}\n", key, value));
                }
            }
            else {
                documentation.push_str("    none\n");
            }

            match module_instance.get_connector_spec() {
                Some(connector_spec) => documentation.push_str(&format!("  connector: {}\n", connector_spec.id)),
                None => documentation.push_str("  connector: none\n"),
            };

            match module_instance.get_display_options().parent_id.as_str() {
                "" => documentation.push_str("  monitor dependency: none (category-level command)\n"),
                parent_id => documentation.push_str(&format!("  monitor dependency: {}\n", parent_id)),
            };

            documentation.push('\n');
        }
        documentation
    }

    pub fn get_connector_module_info(&self) -> String {
        let mut documentation = String::from("Connector modules:\n");

        for (metadata, _) in self.connector_modules.iter() {
            documentation.push_str(&format!("{}:\n", metadata.module_spec));
            documentation.push_str(&format!("  description: {}\n", &metadata.description));
            documentation.push_str("  settings:\n");
            if metadata.settings.is_empty() {
                for (key, value) in metadata.settings.iter() {
                    documentation.push_str(&format!("    {}: {}\n", key, value));
                }
            }
            else {
                documentation.push_str("    none\n");
            }
        }
        documentation
    }

    fn load_modules(&mut self) {
        // Connection modules.
        self.connector_modules = vec![
            (connection::Ssh2::get_metadata(), connection::Ssh2::new_connection_module),
            (connection::Http::get_metadata(), connection::Http::new_connection_module),
            (connection::HttpJwt::get_metadata(), connection::HttpJwt::new_connection_module),
            (connection::LocalCommand::get_metadata(), connection::LocalCommand::new_connection_module),
            (connection::Tcp::get_metadata(), connection::Tcp::new_connection_module),
        ];

        // Monitoring modules.
        self.monitor_modules = vec![
            (monitoring::os::Os::get_metadata(), monitoring::os::Os::new_monitoring_module),
            (monitoring::linux::Package::get_metadata(), monitoring::linux::Package::new_monitoring_module),
            (monitoring::linux::Kernel::get_metadata(), monitoring::linux::Kernel::new_monitoring_module),
            (monitoring::linux::Interface::get_metadata(), monitoring::linux::Interface::new_monitoring_module),
            (monitoring::linux::Uptime::get_metadata(), monitoring::linux::Uptime::new_monitoring_module),
            (monitoring::linux::Load::get_metadata(), monitoring::linux::Load::new_monitoring_module),
            (monitoring::linux::Ram::get_metadata(), monitoring::linux::Ram::new_monitoring_module),
            (monitoring::linux::Who::get_metadata(), monitoring::linux::Who::new_monitoring_module),
            (monitoring::nixos::RebuildGenerations::get_metadata(), monitoring::nixos::RebuildGenerations::new_monitoring_module),
            (monitoring::storage::Filesystem::get_metadata(), monitoring::storage::Filesystem::new_monitoring_module),
            (monitoring::storage::Cryptsetup::get_metadata(),  monitoring::storage::Cryptsetup::new_monitoring_module),
            (monitoring::storage::lvm::LogicalVolume::get_metadata(), monitoring::storage::lvm::LogicalVolume::new_monitoring_module),
            (monitoring::storage::lvm::VolumeGroup::get_metadata(), monitoring::storage::lvm::VolumeGroup::new_monitoring_module),
            (monitoring::storage::lvm::PhysicalVolume::get_metadata(), monitoring::storage::lvm::PhysicalVolume::new_monitoring_module),
            (monitoring::systemd::Service::get_metadata(), monitoring::systemd::Service::new_monitoring_module),
            (monitoring::network::Oping::get_metadata(), monitoring::network::Oping::new_monitoring_module),
            (monitoring::network::Ping::get_metadata(), monitoring::network::Ping::new_monitoring_module),
            (monitoring::network::Ssh::get_metadata(), monitoring::network::Ssh::new_monitoring_module),
            (monitoring::network::TcpConnect::get_metadata(), monitoring::network::TcpConnect::new_monitoring_module),
            (monitoring::network::Routes::get_metadata(), monitoring::network::Routes::new_monitoring_module),
            (monitoring::network::Dns::get_metadata(), monitoring::network::Dns::new_monitoring_module),
            (monitoring::docker::Compose::get_metadata(), monitoring::docker::Compose::new_monitoring_module),
            (monitoring::docker::Containers::get_metadata(), monitoring::docker::Containers::new_monitoring_module),
            (monitoring::docker::Images::get_metadata(), monitoring::docker::Images::new_monitoring_module),

            // Monitoring extension modules.
            (monitoring::docker::ImageUpdates::get_metadata(), monitoring::docker::ImageUpdates::new_monitoring_module),
        ];

        // Command modules.
        self.command_modules = vec![
            (command::linux::Logs::get_metadata(), command::linux::Logs::new_command_module),
            (command::os::Reboot::get_metadata(), command::os::Reboot::new_command_module),
            (command::os::Shutdown::get_metadata(), command::os::Shutdown::new_command_module),
            (command::linux::Shell::get_metadata(), command::linux::Shell::new_command_module),
            (command::linux::packages::Clean::get_metadata(), command::linux::packages::Clean::new_command_module),
            (command::linux::packages::Update::get_metadata(), command::linux::packages::Update::new_command_module),
            (command::linux::packages::UpdateAll::get_metadata(), command::linux::packages::UpdateAll::new_command_module),
            (command::linux::packages::Refresh::get_metadata(), command::linux::packages::Refresh::new_command_module),
            (command::linux::packages::Logs::get_metadata(), command::linux::packages::Logs::new_command_module),
            (command::nixos::RebuildDryrun::get_metadata(), command::nixos::RebuildDryrun::new_command_module),
            (command::nixos::RebuildSwitch::get_metadata(), command::nixos::RebuildSwitch::new_command_module),
            (command::nixos::RebuildBoot::get_metadata(), command::nixos::RebuildBoot::new_command_module),
            (command::nixos::RebuildRollback::get_metadata(), command::nixos::RebuildRollback::new_command_module),
            (command::nixos::CollectGarbage::get_metadata(), command::nixos::CollectGarbage::new_command_module),
            (command::nixos::ChannelUpdate::get_metadata(), command::nixos::ChannelUpdate::new_command_module),
            (command::storage::FileSpaceUsage::get_metadata(), command::storage::FileSpaceUsage::new_command_module),
            (command::storage::lvm::Snapshot::get_metadata(), command::storage::lvm::Snapshot::new_command_module),
            (command::storage::lvm::LVResize::get_metadata(), command::storage::lvm::LVResize::new_command_module),
            (command::storage::lvm::LVRemove::get_metadata(), command::storage::lvm::LVRemove::new_command_module),
            (command::storage::lvm::LVRefresh::get_metadata(), command::storage::lvm::LVRefresh::new_command_module),
            (command::docker::Restart::get_metadata(), command::docker::Restart::new_command_module),
            (command::docker::Inspect::get_metadata(), command::docker::Inspect::new_command_module),
            (command::docker::Shell::get_metadata(), command::docker::Shell::new_command_module),
            (command::docker::image::Remove::get_metadata(), command::docker::image::Remove::new_command_module),
            (command::docker::image::Prune::get_metadata(), command::docker::image::Prune::new_command_module),
            (command::docker::image::RemoteTags::get_metadata(), command::docker::image::RemoteTags::new_command_module),
            (command::docker::compose::Edit::get_metadata(), command::docker::compose::Edit::new_command_module),
            (command::docker::compose::Pull::get_metadata(), command::docker::compose::Pull::new_command_module),
            (command::docker::compose::Up::get_metadata(), command::docker::compose::Up::new_command_module),
            (command::docker::compose::Start::get_metadata(), command::docker::compose::Start::new_command_module),
            (command::docker::compose::Stop::get_metadata(), command::docker::compose::Stop::new_command_module),
            (command::docker::compose::Shell::get_metadata(), command::docker::compose::Shell::new_command_module),
            (command::docker::compose::Logs::get_metadata(), command::docker::compose::Logs::new_command_module),
            (command::docker::compose::Build::get_metadata(), command::docker::compose::build::Build::new_command_module),
            (command::systemd::service::Start::get_metadata(), command::systemd::service::Start::new_command_module),
            (command::systemd::service::Stop::get_metadata(), command::systemd::service::Stop::new_command_module),
            (command::systemd::service::Mask::get_metadata(), command::systemd::service::Mask::new_command_module),
            (command::systemd::service::Unmask::get_metadata(), command::systemd::service::Unmask::new_command_module),
            (command::systemd::service::Logs::get_metadata(), command::systemd::service::Logs::new_command_module),
            (command::network::SocketListen::get_metadata(), command::network::SocketListen::new_command_module),
            (command::network::SocketTcp::get_metadata(), command::network::SocketTcp::new_command_module),
        ];

        self.validate_modules();
        self.connector_modules.iter().map(|(metadata, _)| metadata).for_each(|metadata| log::debug!("Loaded connector module: {}", metadata.module_spec.id));
        self.monitor_modules.iter().map(|(metadata, _)| metadata).for_each(|metadata| log::debug!("Loaded monitoring module: {}", metadata.module_spec.id));
        self.command_modules.iter().map(|(metadata, _)| metadata).for_each(|metadata| log::debug!("Loaded command module: {}", metadata.module_spec.id));

        log::info!("Loaded {} command modules, {} monitoring modules and {} connector modules",
                   self.command_modules.len(), self.monitor_modules.len(), self.connector_modules.len());

    }

}
