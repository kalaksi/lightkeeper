extern crate qmetaobject;

use qmetaobject::*;
use std::str::FromStr;

use crate::{
    configuration::{self, Configuration, Groups, HostSettings, Hosts}, module::{Metadata, ModuleType}
};


#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct ConfigManagerModel {
    base: qt_base_class!(trait QObject),

    //
    // Signals
    //
    // TODO: use this? implement in some other way?
    fileError: qt_signal!(config_dir: QString, error_message: QString),

    //
    // Common
    //
    isSandboxed: qt_method!(fn(&self) -> bool),
    isDevBuild: qt_method!(fn(&self) -> bool),
    getCurrentWorkDir: qt_method!(fn(&self) -> QString),

    //
    // Preferences
    //
    getPreferences: qt_method!(fn(&self) -> QVariantMap),
    setPreferences: qt_method!(fn(&self, preferences: QVariantMap)),

    //
    // Certificate monitoring
    //
    getCertificateMonitors: qt_method!(fn(&self) -> QStringList),
    addCertificateMonitor: qt_method!(fn(&self, address: QString)),
    removeCertificateMonitor: qt_method!(fn(&self, address: QString)),

    //
    // Host configuration
    //
    addHost: qt_method!(fn(&self, host_name: QString)),
    removeHost: qt_method!(fn(&self, host_name: QString)),
    /// Returns HostSettings as JSON string, since it doesn't seem to be possible to return custom QObjects directly.
    getHostSettings: qt_method!(fn(&self, host_name: QString) -> QString),
    setHostSettings: qt_method!(fn(&self, old_host_name: QString, new_host_name: QString, host_settings_json: QString)),
    beginHostConfiguration: qt_method!(fn(&self)),
    cancelHostConfiguration: qt_method!(fn(&self)),
    endHostConfiguration: qt_method!(fn(&self)),

    getSelectedGroups: qt_method!(fn(&self, host_name: QString) -> QStringList),
    getAvailableGroups: qt_method!(fn(&self, host_name: QString) -> QStringList),
    addHostToGroup: qt_method!(fn(&self, host_name: QString, group_name: QString)),
    removeHostFromGroup: qt_method!(fn(&self, host_name: QString, group_name: QString)),

    //
    // Group configuration
    //
    beginGroupConfiguration: qt_method!(fn(&self)),
    cancelGroupConfiguration: qt_method!(fn(&self)),
    endGroupConfiguration: qt_method!(fn(&self)),

    get_all_groups: qt_method!(fn(&self) -> QStringList),
    addGroup: qt_method!(fn(&self, group_name: QString)),
    remove_group: qt_method!(fn(&self, group_name: QString)),
    get_all_module_settings: qt_method!(fn(&self, module_type: QString, module_id: QString) -> QVariantMap),

    compareToDefault: qt_method!(fn(&self, group_name: QString) -> QStringList),
    ignoreFromConfigHelper: qt_method!(fn(&self, group_name: QString, commands: QStringList, monitors: QStringList, connectors: QStringList)),

    //
    // Group configuration: connectors
    //
    getUnselectedConnectors: qt_method!(fn(&self, group_name: QString) -> QStringList),
    get_connector_description: qt_method!(fn(&self, connector_name: QString) -> QString),
    get_group_connectors: qt_method!(fn(&self, group_name: QString) -> QStringList),
    addGroupConnector: qt_method!(fn(&self, group_name: QString, connector_name: QString)),
    get_group_connector_settings_keys: qt_method!(fn(&self, group_name: QString, connector_name: QString) -> QStringList),
    get_group_connector_setting: qt_method!(fn(&self, group_name: QString, connector_name: QString, setting_key: QString) -> QString),
    set_group_connector_setting: qt_method!(fn(&self, group_name: QString, connector_name: QString, setting_key: QString, setting_value: QString)),
    remove_group_connector: qt_method!(fn(&self, group_name: QString, connector_name: QString)),

    //
    // Group configuration: monitors
    //
    // NOTE: currently "unset" acts as a special value for indicating if a setting is unset.
    getUnselectedMonitors: qt_method!(fn(&self, group_name: QString) -> QStringList),
    get_monitor_description: qt_method!(fn(&self, monitor_name: QString) -> QString),
    get_group_monitors: qt_method!(fn(&self, group_name: QString) -> QStringList),
    addGroupMonitor: qt_method!(fn(&self, group_name: QString, monitor_name: QString)),
    remove_group_monitor: qt_method!(fn(&self, group_name: QString, monitor_name: QString)),
    // These 2 are currently not really used.
    get_group_monitor_enabled: qt_method!(fn(&self, group_name: QString, monitor_name: QString) -> QString),
    toggle_group_monitor_enabled: qt_method!(fn(&self, group_name: QString, monitor_name: QString)),
    get_group_monitor_settings_keys: qt_method!(fn(&self, group_name: QString, monitor_name: QString) -> QStringList),
    get_group_monitor_setting: qt_method!(fn(&self, group_name: QString, monitor_name: QString, setting_key: QString) -> QString),
    set_group_monitor_setting: qt_method!(fn(&self, group_name: QString, monitor_name: QString, setting_key: QString, setting_value: QString)),

    //
    // Group configuration: commands
    //
    getUnselectedCommands: qt_method!(fn(&self, group_name: QString) -> QStringList),
    get_command_description: qt_method!(fn(&self, command_name: QString) -> QString),
    get_group_commands: qt_method!(fn(&self, group_name: QString) -> QStringList),
    addGroupCommand: qt_method!(fn(&self, group_name: QString, command_name: QString)),
    remove_group_command: qt_method!(fn(&self, group_name: QString, command_name: QString)),
    get_group_command_settings_keys: qt_method!(fn(&self, group_name: QString, command_name: QString) -> QStringList),
    get_group_command_setting: qt_method!(fn(&self, group_name: QString, command_name: QString, setting_key: QString) -> QString),
    set_group_command_setting: qt_method!(fn(&self, group_name: QString, command_name: QString, setting_key: QString, setting_value: QString)),


    config_dir: String,
    main_config: Configuration,
    hosts_config: Hosts,
    hosts_config_backup: Option<Hosts>,
    groups_config: Groups,
    groups_config_backup: Option<Groups>,
    module_metadatas: Vec<Metadata>,
}

#[allow(non_snake_case)]
impl ConfigManagerModel {
    pub fn new(config_dir: String,
               mut main_config: Configuration,
               hosts_config: Hosts,
               mut groups_config: Groups,
               module_metadatas: Vec<Metadata>) -> Self {
        
        let mut hosts_config = hosts_config;
        // Sort host groups alphabetically.
        for host in hosts_config.hosts.values_mut() {
            host.groups.sort_by_key(|key| key.to_lowercase());
        }

        if Configuration::is_schema_outdated(main_config.schema_version) {
            Configuration::upgrade_schema(&mut main_config, &mut groups_config);

            // If writes fail, log errors but continue nevertheless.
            if let Err(error) = Configuration::write_main_config(&config_dir, &main_config) {
                ::log::error!("Failed to write main configuration: {}", error);
            }
            if let Err(error) = Configuration::write_groups_config(&config_dir, &groups_config) {
                ::log::error!("Failed to write groups configuration: {}", error);
            }
        }

        ConfigManagerModel {
            config_dir: config_dir,
            main_config: main_config,
            hosts_config: hosts_config,
            groups_config: groups_config,
            module_metadatas: module_metadatas,
            ..Default::default()
        }
    }

    pub fn reload_configuration(&mut self) -> Result<(Configuration, Hosts), String> {
        ::log::info!("Reloading configuration...");
        match Configuration::read(&self.config_dir) {
            Ok((main_config, hosts_config, groups_config)) => {
                self.main_config = main_config.clone();
                self.hosts_config = hosts_config.clone();
                self.groups_config = groups_config;
                Ok((main_config, hosts_config))
            },
            Err(error) => Err(error.to_string())
        }
    }


    fn getPreferences(&self) -> QVariantMap {
        let mut preferences = QVariantMap::default();
        preferences.insert("refreshHostsOnStart".into(), self.main_config.preferences.refresh_hosts_on_start.into());
        preferences.insert("useRemoteEditor".into(), self.main_config.preferences.use_remote_editor.into());
        preferences.insert("remoteTextEditor".into(), QString::from(self.main_config.preferences.remote_text_editor.clone()).into());
        preferences.insert("sudoRemoteEditor".into(), self.main_config.preferences.sudo_remote_editor.into());
        preferences.insert("textEditor".into(), QString::from(self.main_config.preferences.text_editor.clone()).into());
        preferences.insert("terminal".into(), QString::from(self.main_config.preferences.terminal.clone()).into());
        preferences.insert("terminalArgs".into(), QString::from(self.main_config.preferences.terminal_args.join(" ")).into());
        preferences.insert("closeToTray".into(), self.main_config.preferences.close_to_tray.into());
        preferences.insert("showStatusBar".into(), self.main_config.display_options.show_status_bar.into());
        preferences
    }

    fn setPreferences(&mut self, preferences: QVariantMap) {
        self.main_config.preferences.refresh_hosts_on_start = preferences.value("refreshHostsOnStart".into(), false.into()).to_bool();
        self.main_config.preferences.use_remote_editor = preferences.value("useRemoteEditor".into(), false.into()).to_bool();
        self.main_config.preferences.remote_text_editor = preferences.value("remoteTextEditor".into(), QString::from("vim").into()).to_qbytearray().to_string();
        self.main_config.preferences.sudo_remote_editor = preferences.value("sudoRemoteEditor".into(), false.into()).to_bool();
        self.main_config.preferences.text_editor = preferences.value("textEditor".into(), QString::from("kate").into()).to_qbytearray().to_string();
        self.main_config.preferences.terminal = preferences.value("terminal".into(), QString::from("xterm").into()).to_qbytearray().to_string();
        self.main_config.preferences.terminal_args = preferences.value("terminalArgs".into(), QString::from("-e").into()).to_qbytearray().to_string()
                                                                .split(' ').map(|arg| arg.to_string()).collect();
        self.main_config.preferences.close_to_tray = preferences.value("closeToTray".into(), true.into()).to_bool();

        self.main_config.display_options.show_status_bar = preferences.value("showStatusBar".into(), true.into()).to_bool();

        if let Err(error) = Configuration::write_main_config(&self.config_dir, &self.main_config) {
            self.fileError(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
        }
    }

    fn addHost(&mut self, host_name: QString) {
        let host_name = host_name.to_string();
        let config = HostSettings {
            groups: vec![
                String::from("defaults"),
                String::from("linux"),
                String::from("systemd-service"),
                String::from("docker"),
                String::from("docker-compose"),
            ],
            ..Default::default()
        };
        self.hosts_config.hosts.insert(host_name, config);
    }

    fn removeHost(&mut self, host_name: QString) {
        let host_name = host_name.to_string();
        ::log::info!("Removing host {}", host_name);
        self.hosts_config.hosts.remove(&host_name);
    }

    fn getCertificateMonitors(&self) -> QStringList {
        QStringList::from_iter(self.hosts_config.certificate_monitors.clone())
    }

    fn addCertificateMonitor(&mut self, domain: QString) {
        if self.hosts_config.certificate_monitors.iter().any(|monitor_domain| monitor_domain == &domain.to_string()) {
            return;
        }

        self.hosts_config.certificate_monitors.push(domain.to_string());

        if let Err(error) = Configuration::write_hosts_config(&self.config_dir, &self.hosts_config) {
            self.fileError(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
        }
    }

    fn removeCertificateMonitor(&mut self, domain: QString) {
        self.hosts_config.certificate_monitors.retain(|monitor_domain| monitor_domain != &domain.to_string());

        if let Err(error) = Configuration::write_hosts_config(&self.config_dir, &self.hosts_config) {
            self.fileError(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
        }
    }

    fn isSandboxed(&self) -> bool {
        self.main_config.preferences.use_sandbox_mode
    }

    fn isDevBuild(&self) -> bool {
        cfg!(debug_assertions)
    }

    fn getCurrentWorkDir(&self) -> QString {
        QString::from(std::env::current_dir().unwrap().to_string_lossy().to_string())
    }

    /// Updates preferences.use_sandbox_mode. Returns true if value was changed and was written to config.
    pub fn setSandboxed(&mut self, use_sandbox_mode: bool) -> bool {
        if self.main_config.preferences.use_sandbox_mode != use_sandbox_mode {
            self.main_config.preferences.use_sandbox_mode = use_sandbox_mode;
            if use_sandbox_mode {
                self.main_config.preferences.text_editor = configuration::INTERNAL.to_string();
                self.main_config.preferences.terminal = configuration::INTERNAL.to_string();
                self.main_config.preferences.terminal_args = Vec::new();
            }

            if let Err(error) = Configuration::write_main_config(&self.config_dir, &self.main_config) {
                self.fileError(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
                false
            }
            else {
                true
            }
        }
        else {
            false
        }
    }

    fn beginHostConfiguration(&mut self) {
        self.hosts_config_backup = Some(self.hosts_config.clone());
    }

    fn cancelHostConfiguration(&mut self) {
        self.hosts_config = self.hosts_config_backup.take().unwrap();
    }

    fn endHostConfiguration(&mut self) {
        self.hosts_config_backup = None;
        if let Err(error) = Configuration::write_hosts_config(&self.config_dir, &self.hosts_config) {
            self.fileError(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
        }
    }

    fn beginGroupConfiguration(&mut self) {
        self.groups_config_backup = Some(self.groups_config.clone());
    }

    fn cancelGroupConfiguration(&mut self) {
        self.groups_config = self.groups_config_backup.take().unwrap();
    }

    fn endGroupConfiguration(&mut self) {
        self.groups_config_backup = None;
        if let Err(error) = Configuration::write_groups_config(&self.config_dir, &self.groups_config) {
            self.fileError(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
        }
    }

    fn getHostSettings(&self, host_name: QString) -> QString {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).unwrap_or(&Default::default()).clone();

        QString::from(serde_json::to_string(&host_settings).unwrap())
    }

    fn setHostSettings(&mut self, old_host_name: QString, new_host_name: QString, host_settings_json: QString) {
        let old_host_name = old_host_name.to_string();
        let new_host_name = new_host_name.to_string();
        let new_settings: HostSettings = serde_json::from_str(&host_settings_json.to_string()).unwrap();

        if old_host_name != new_host_name {
            let host_config = self.hosts_config.hosts.remove(&old_host_name).unwrap();
            self.hosts_config.hosts.insert(new_host_name.clone(), host_config);
        }

        let host_config = self.hosts_config.hosts.get_mut(&new_host_name).unwrap();
        host_config.address = new_settings.address;
        host_config.fqdn = new_settings.fqdn;
        host_config.overrides = new_settings.overrides;
    }

    fn get_all_groups(&self) -> QStringList {
        let mut all_groups = self.groups_config.groups.keys().cloned().collect::<Vec<String>>();
        all_groups.sort();
        all_groups.into_iter().map(QString::from).collect()
    }

    fn addGroup(&mut self, group_name: QString) {
        let group_name = group_name.to_string();
        self.groups_config.groups.insert(group_name, Default::default());
    }

    fn remove_group(&mut self, group_name: QString) {
        let group_name = group_name.to_string();
        self.groups_config.groups.remove(&group_name);
    }

    fn getSelectedGroups(&self, host_name: QString) -> QStringList {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).cloned().unwrap_or_default();
        let groups_sorted = {
            let mut groups = host_settings.groups.clone();
            groups.sort_by_key(|key| key.to_lowercase());
            groups
        };

        groups_sorted.into_iter().map(QString::from).collect()
    }

    fn getAvailableGroups(&self, host_name: QString) -> QStringList {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).cloned().unwrap_or_default();

        let all_groups = self.groups_config.groups.keys().collect::<Vec<&String>>();
        let available_groups = all_groups.iter()
            .filter(|group| !host_settings.groups.contains(group))
            .map(|group| group.to_string())
            .collect::<Vec<String>>();

        available_groups.into_iter().map(QString::from).collect()
    }

    fn addHostToGroup(&mut self, host_name: QString, group_name: QString) {
        let host_name = host_name.to_string();
        let group_name = group_name.to_string();
        let host_settings = self.hosts_config.hosts.get_mut(&host_name).unwrap();

        host_settings.groups.push(group_name);
    }

    fn removeHostFromGroup(&mut self, host_name: QString, group_name: QString) {
        let host_name = host_name.to_string();
        let group_name = group_name.to_string();
        let host_settings = self.hosts_config.hosts.get_mut(&host_name).unwrap();

        host_settings.groups.retain(|group| group != &group_name);
    }

    fn getUnselectedMonitors(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_monitors = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().monitors;

        let all_monitors = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.module_type == ModuleType::Monitor)
            .map(|metadata| metadata.module_spec.id.clone())
            .collect::<Vec<String>>();

        let mut unselected_monitors = all_monitors.iter()
            .filter(|monitor| !group_monitors.contains_key(*monitor))
            .cloned()
            .collect::<Vec<String>>();

        unselected_monitors.sort();
        unselected_monitors.into_iter().map(QString::from).collect()
    }

    fn get_monitor_description(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == ModuleType::Monitor)
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    fn get_group_monitors(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_monitors = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().monitors;

        let mut group_monitors_keys = group_monitors.into_keys().collect::<Vec<String>>();
        group_monitors_keys.sort_by_key(|key| key.to_lowercase());
        group_monitors_keys.into_iter().map(QString::from).collect()
    }

    fn addGroupMonitor(&mut self, group_name: QString, monitor_name: QString) {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().monitors.insert(monitor_name, Default::default());
    }

    fn remove_group_monitor(&mut self, group_name: QString, monitor_name: QString) {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().monitors.remove(&monitor_name);
    }

    fn get_group_monitor_enabled(&self, group_name: QString, monitor_name: QString) -> QString {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();

        self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                          .monitors.get(&monitor_name).cloned().unwrap_or_default()
                          .enabled.unwrap_or(true).to_string().into()
    }

    fn toggle_group_monitor_enabled(&mut self, group_name: QString, monitor_name: QString) {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();

        let group_monitor_settings = self.groups_config.groups.get_mut(&group_name).unwrap()
                                                       .monitors.get_mut(&monitor_name).unwrap();

        if let Some(enabled) = group_monitor_settings.enabled {
            group_monitor_settings.enabled = Some(!enabled);
        } else {
            group_monitor_settings.enabled = Some(false);
        }
    }

    fn get_group_monitor_settings_keys(&self, group_name: QString, monitor_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        let group_monitor_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                       .monitors.get(&monitor_name).cloned().unwrap_or_default().settings;

        let mut group_monitor_settings_keys = group_monitor_settings.into_keys().collect::<Vec<String>>();
        group_monitor_settings_keys.sort_by_key(|key| key.to_lowercase());
        group_monitor_settings_keys.into_iter().map(QString::from).collect()
    }

    fn get_group_monitor_setting(&self, group_name: QString, monitor_name: QString, setting_key: QString) -> QString {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        let setting_key = setting_key.to_string();
        let group_monitor_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                       .monitors.get(&monitor_name).cloned().unwrap_or_default().settings;

        QString::from(group_monitor_settings.get(&setting_key).cloned().unwrap_or(String::from("unset")))
    }

    fn set_group_monitor_setting(&mut self, group_name: QString, monitor_name: QString, setting_key: QString, setting_value: QString) {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        let setting_key = setting_key.to_string();
        let setting_value = setting_value.to_string();

        let group_monitor_settings = self.groups_config.groups.get_mut(&group_name).unwrap()
                                                       .monitors.get_mut(&monitor_name).unwrap();
        if setting_value == "unset" {
            group_monitor_settings.settings.remove(&setting_key);
        }
        else {
            group_monitor_settings.settings.insert(setting_key, setting_value);
        }
    }

    fn getUnselectedConnectors(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_connectors = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().connectors;

        let all_connectors = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.module_type == ModuleType::Connector)
            .map(|metadata| metadata.module_spec.id.clone())
            .collect::<Vec<String>>();

        let mut unselected_connectors = all_connectors.iter()
            .filter(|connector| !group_connectors.contains_key(*connector))
            .cloned()
            .collect::<Vec<String>>();

        unselected_connectors.sort();
        unselected_connectors.into_iter().map(QString::from).collect()
    }

    fn get_connector_description(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == ModuleType::Connector)
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    fn get_group_connectors(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_connectors = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().connectors;

        let mut group_connectors_keys = group_connectors.into_keys().collect::<Vec<String>>();
        group_connectors_keys.sort_by_key(|key| key.to_lowercase());
        group_connectors_keys.into_iter().map(QString::from).collect()
    }

    fn addGroupConnector(&mut self, group_name: QString, connector_name: QString) {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().connectors.insert(connector_name, Default::default());
    }

    fn get_group_connector_settings_keys(&self, group_name: QString, connector_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        let group_connector_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                         .connectors.get(&connector_name).cloned().unwrap_or_default().settings;

        let mut group_connector_settings_keys = group_connector_settings.into_keys().collect::<Vec<String>>();
        group_connector_settings_keys.sort_by_key(|key| key.to_lowercase());
        group_connector_settings_keys.into_iter().map(QString::from).collect()
    }

    fn get_group_connector_setting(&self, group_name: QString, connector_name: QString, setting_key: QString) -> QString {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        let setting_key = setting_key.to_string();
        let group_connector_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                         .connectors.get(&connector_name).cloned().unwrap_or_default().settings;

        QString::from(group_connector_settings.get(&setting_key).cloned().unwrap_or(String::from("unset")))
    }

    fn set_group_connector_setting(&mut self, group_name: QString, connector_name: QString, setting_key: QString, setting_value: QString) {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        let setting_key = setting_key.to_string();
        let setting_value = setting_value.to_string();

        let group_connector_settings = self.groups_config.groups.get_mut(&group_name).unwrap()
                                                         .connectors.get_mut(&connector_name).unwrap();

        if setting_value == "unset" {
            group_connector_settings.settings.remove(&setting_key);
        }
        else {
            group_connector_settings.settings.insert(setting_key, setting_value);
        }
    }

    fn remove_group_connector(&mut self, group_name: QString, connector_name: QString) {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().connectors.remove(&connector_name);
    }

    fn getUnselectedCommands(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_commands = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().commands;

        let all_commands = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.module_type == ModuleType::Command)
            .map(|metadata| metadata.module_spec.id.clone())
            .collect::<Vec<String>>();

        let mut unselected_commands = all_commands.iter()
            .filter(|command| !group_commands.contains_key(*command))
            .cloned()
            .collect::<Vec<String>>();

        unselected_commands.sort();
        unselected_commands.into_iter().map(QString::from).collect()
    }

    fn get_command_description(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == ModuleType::Command)
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    fn get_group_commands(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_commands = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().commands;

        let mut group_commands_keys = group_commands.into_keys().collect::<Vec<String>>();
        group_commands_keys.sort_by_key(|key| key.to_lowercase());

        group_commands_keys.into_iter().map(QString::from).collect()
    }

    fn addGroupCommand(&mut self, group_name: QString, command_name: QString) {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().commands.insert(command_name, Default::default());
    }

    fn remove_group_command(&mut self, group_name: QString, command_name: QString) {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().commands.remove(&command_name);
    }

    fn get_group_command_settings_keys(&self, group_name: QString, command_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        let group_command_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                       .commands.get(&command_name).cloned().unwrap_or_default().settings;

        let mut group_command_settings_keys = group_command_settings.into_keys().collect::<Vec<String>>();
        group_command_settings_keys.sort_by_key(|key| key.to_lowercase());
        group_command_settings_keys.into_iter().map(QString::from).collect()
    }

    fn get_group_command_setting(&self, group_name: QString, command_name: QString, setting_key: QString) -> QString {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        let setting_key = setting_key.to_string();
        let group_command_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                         .commands.get(&command_name).cloned().unwrap_or_default().settings;

        QString::from(group_command_settings.get(&setting_key).cloned().unwrap_or(String::from("unset")))
    }

    fn set_group_command_setting(&mut self, group_name: QString, command_name: QString, setting_key: QString, setting_value: QString) {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        let setting_key = setting_key.to_string();
        let setting_value = setting_value.to_string();

        let group_command_settings = self.groups_config.groups.get_mut(&group_name).unwrap()
                                                         .commands.get_mut(&command_name).unwrap();

        if setting_value == "unset" {
            group_command_settings.settings.remove(&setting_key);
        }
        else {
            group_command_settings.settings.insert(setting_key, setting_value);
        }
    }

    fn get_all_module_settings(&self, module_type: QString, module_id: QString) -> QVariantMap {
        let module_id = module_id.to_string();
        let module_type = ModuleType::from_str(module_type.to_string().as_str()).unwrap();
        let module_settings = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_id && metadata.module_spec.module_type == module_type)
            .map(|metadata| metadata.settings.clone())
            .next().unwrap_or_default();

        let mut module_settings_keys = module_settings.keys().collect::<Vec<&String>>();
        module_settings_keys.sort_by(|&a, &b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut result = QVariantMap::default();
        for setting_key in module_settings_keys {
            let qvariant = module_settings.get(setting_key).map(|key| QString::from(key.clone())).unwrap_or_default();
            result.insert(setting_key.clone().into(), qvariant.into());
        }
        result
    }

    fn compareToDefault(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let mut result = QStringList::default();

        let group_settings = match self.groups_config.groups.get(&group_name) {
            Some(group_settings) => group_settings,
            // Don't return anything if group doesn't exist.
            None => return result,
        };
        let config_helper_data = &group_settings.config_helper;
        let default_groups = configuration::get_default_config_groups();
        let default_settings = default_groups.groups.get(&group_name).unwrap();

        let new_commands = default_settings.commands.keys()
            .filter(|id| !group_settings.commands.contains_key(*id) && !config_helper_data.ignored_commands.contains(*id))
            // Will return empty description if not found.
            .map(|id| (id, self.get_command_description(QString::from(id.clone()))))
            .filter(|(_, description)| !description.is_empty())
            .collect::<Vec<_>>();

        let new_monitors = default_settings.monitors.keys()
            .filter(|id| !group_settings.monitors.contains_key(*id) && !config_helper_data.ignored_monitors.contains(*id))
            // Will return empty description if not found.
            .map(|id| (id, self.get_monitor_description(QString::from(id.clone()))))
            .filter(|(_, description)| !description.is_empty())
            .collect::<Vec<_>>();

        let new_connectors = default_settings.connectors.keys()
            .filter(|id| !group_settings.connectors.contains_key(*id) && !config_helper_data.ignored_connectors.contains(*id))
            // Will return empty description if not found.
            .map(|id| (id, self.get_connector_description(QString::from(id.clone()))))
            .filter(|(_, description)| !description.is_empty())
            .collect::<Vec<_>>();

        for (id, description) in new_commands {
            result.push(QString::from(format!("Command: {},{}", id, description)));
        }
        for (id, description) in new_monitors {
            result.push(QString::from(format!("Monitor: {},{}", id, description)));
        }
        for (id, description) in new_connectors {
            result.push(QString::from(format!("Connector: {},{}", id, description)));
        }
        result
    }

    fn ignoreFromConfigHelper(&mut self, group_name: QString, commands: QStringList, monitors: QStringList, connectors: QStringList) {
        let group_name = group_name.to_string();
        if let Some(group_settings) = self.groups_config.groups.get_mut(&group_name) {
            group_settings.config_helper.ignored_commands = commands.into_iter().map(ToString::to_string).collect();
            group_settings.config_helper.ignored_monitors = monitors.into_iter().map(ToString::to_string).collect();
            group_settings.config_helper.ignored_connectors = connectors.into_iter().map(ToString::to_string).collect();
        }
    }
}