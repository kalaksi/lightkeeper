/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

extern crate qmetaobject;

use qmetaobject::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

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
    hostConfigurationChanged: qt_signal!(),

    //
    // Common
    //
    showStatusBar: qt_property!(bool; READ showStatusBar),
    // Called only from QML.
    showCharts: qt_property!(bool; READ showCharts WRITE setShowCharts),
    showInfoNotifications: qt_property!(bool; READ showInfoNotifications),
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
    updateHostGroups: qt_method!(fn(&self, host_name: QString, groups: QStringList)),

    //
    // Custom commands
    //
    getCustomCommands: qt_method!(fn(&self, host_name: QString) -> QStringList),
    updateCustomCommands: qt_method!(fn(&self, host_name: QString, custom_commands_json: QStringList)),

    //
    // Group configuration
    //
    updateGroupModules: qt_method!(fn(&self,
        group_id: QString,
        connector_settings: QString,
        monitor_settings: QString,
        command_settings: QString
    )),
    writeGroupConfiguration: qt_method!(fn(&self)),

    get_all_groups: qt_method!(fn(&self) -> QStringList),
    addGroup: qt_method!(fn(&self, group_id: QString)),
    removeGroup: qt_method!(fn(&self, group_id: QString)),
    compareToDefault: qt_method!(fn(&self, group_name: QString) -> QStringList),
    ignoreFromConfigHelper: qt_method!(fn(&self, group_name: QString, commands: QStringList, monitors: QStringList, connectors: QStringList)),

    //
    // Group configuration: connectors
    //
    getUnselectedConnectorIds: qt_method!(fn(&self, selected_groups: QStringList) -> QStringList),
    getGroupConnectorIds: qt_method!(fn(&self, group_name: QString) -> QStringList),
    getConnectorDescription: qt_method!(fn(&self, connector_name: QString) -> QString),
    addGroupConnector: qt_method!(fn(&self, group_name: QString, connector_name: QString)),
    getGroupConnectorSettings: qt_method!(fn(&self, group_id: QString, module_id: QString) -> QStringList),
    getEffectiveConnectorSettings: qt_method!(fn(&self, host_id: QString, grouplist: QStringList) -> QString),

    //
    // Group configuration: monitors
    //
    getUnselectedMonitorIds: qt_method!(fn(&self, selected_groups: QStringList) -> QStringList),
    getMonitorDescription: qt_method!(fn(&self, monitor_name: QString) -> QString),
    getGroupMonitorIds: qt_method!(fn(&self, group_name: QString) -> QStringList),
    addGroupMonitor: qt_method!(fn(&self, group_name: QString, monitor_name: QString)),
    getGroupMonitorSettings: qt_method!(fn(&self, group_id: QString, module_id: QString) -> QStringList),
    getEffectiveMonitorSettings: qt_method!(fn(&self, host_id: QString, grouplist: QStringList) -> QString),

    //
    // Group configuration: commands
    //
    getUnselectedCommandIds: qt_method!(fn(&self, selected_groups: QStringList) -> QStringList),
    getCommandDescription: qt_method!(fn(&self, command_name: QString) -> QString),
    getGroupCommandIds: qt_method!(fn(&self, group_name: QString) -> QStringList),
    addGroupCommand: qt_method!(fn(&self, group_name: QString, command_name: QString)),
    getGroupCommandSettings: qt_method!(fn(&self, group_id: QString, module_id: QString) -> QStringList),
    getEffectiveCommandSettings: qt_method!(fn(&self, host_id: QString, grouplist: QStringList) -> QString),


    config_dir: String,
    main_config: Configuration,
    hosts_config: Hosts,
    hosts_config_backup: Option<Hosts>,
    groups_config: Groups,
    module_metadatas: Vec<Metadata>,
}

#[allow(non_snake_case)]
impl ConfigManagerModel {
    pub fn new(config_dir: String,
               mut main_config: Configuration,
               hosts_config: Hosts,
               mut groups_config: Groups,
               module_metadatas: Vec<Metadata>) -> Self {
        
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
        preferences.insert("showMonitorNotifications".into(), self.main_config.preferences.show_monitor_notifications.into());

        preferences.insert("showStatusBar".into(), self.main_config.display_options.show_status_bar.into());
        preferences.insert("showCharts".into(), self.main_config.preferences.show_charts.into());
        preferences
    }

    fn setPreferences(&mut self, preferences: QVariantMap) {
        self.main_config.preferences.refresh_hosts_on_start = preferences.value("refreshHostsOnStart".into(), false.into()).to_bool();
        self.main_config.preferences.use_remote_editor = preferences.value("useRemoteEditor".into(), false.into()).to_bool();
        self.main_config.preferences.remote_text_editor = preferences.value("remoteTextEditor".into(), QString::from("vim").into()).to_qbytearray().to_string();
        self.main_config.preferences.sudo_remote_editor = preferences.value("sudoRemoteEditor".into(), false.into()).to_bool();
        let text_editor = preferences.value("textEditor".into(), QString::from("kate").into()).to_qbytearray().to_string();
        // In sandboxed mode, only allow "internal" or "internal-simple"
        if self.main_config.preferences.use_sandbox_mode 
            && text_editor != configuration::INTERNAL 
            && text_editor != configuration::INTERNAL_SIMPLE {

            self.main_config.preferences.text_editor = configuration::INTERNAL.to_string();
        } else {
            self.main_config.preferences.text_editor = text_editor;
        }
        self.main_config.preferences.terminal = preferences.value("terminal".into(), QString::from("xterm").into()).to_qbytearray().to_string();
        self.main_config.preferences.terminal_args = preferences.value("terminalArgs".into(), QString::from("-e").into()).to_qbytearray().to_string()
                                                                .split(' ').map(|arg| arg.to_string()).collect();
        self.main_config.preferences.close_to_tray = preferences.value("closeToTray".into(), true.into()).to_bool();
        self.main_config.preferences.show_monitor_notifications = preferences.value("showMonitorNotifications".into(), true.into()).to_bool();
        self.main_config.preferences.show_charts = preferences.value("showCharts".into(), false.into()).to_bool();

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

    fn showStatusBar(&self) -> bool {
        self.main_config.display_options.show_status_bar
    }

    fn showCharts(&self) -> bool {
        self.main_config.preferences.show_charts
    }

    fn setShowCharts(&mut self, show_charts: bool) {
        if self.main_config.preferences.show_charts != show_charts {
            self.main_config.preferences.show_charts = show_charts;

            if let Err(error) = Configuration::write_main_config(&self.config_dir, &self.main_config) {
                self.fileError(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
            }
        }
    }

    fn showInfoNotifications(&self) -> bool {
        self.main_config.preferences.show_monitor_notifications
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
                if self.main_config.preferences.text_editor != configuration::INTERNAL 
                    && self.main_config.preferences.text_editor != configuration::INTERNAL_SIMPLE {

                    self.main_config.preferences.text_editor = configuration::INTERNAL.to_string();
                }
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

    // TODO: this is wrong way around, shouldn't modify config directly. 
    // Maybe ditch the state in here and update everything in batch from UI like it's done like with updateCustomCommands()?
    // OTOH, doing less in JS is better...
    fn endHostConfiguration(&mut self) {
        self.hosts_config_backup = None;
        if let Err(error) = Configuration::write_hosts_config(&self.config_dir, &self.hosts_config) {
            self.fileError(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
        }
        self.hostConfigurationChanged();
    }

    fn writeGroupConfiguration(&mut self) {
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


        if let Some(host_config) = self.hosts_config.hosts.get_mut(&new_host_name) {
            // Preserve custom commands. They are not configured in the host settings dialog.
            let custom_commands = host_config.overrides.custom_commands.clone();

            host_config.address = new_settings.address;
            host_config.fqdn = new_settings.fqdn;
            host_config.overrides = new_settings.overrides;

            host_config.overrides.custom_commands = custom_commands;
        }
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

    fn removeGroup(&mut self, group_name: QString) {
        let group_name = group_name.to_string();
        self.groups_config.groups.remove(&group_name);
    }

    fn getSelectedGroups(&self, host_name: QString) -> QStringList {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).cloned().unwrap_or_default();

        host_settings.groups.into_iter().map(QString::from).collect()
    }

    fn getAvailableGroups(&self, host_name: QString) -> QStringList {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).cloned().unwrap_or_default();

        let all_groups = self.groups_config.groups.keys().collect::<Vec<&String>>();
        let mut available_groups = all_groups.iter()
            .filter(|group| !host_settings.groups.contains(group))
            .map(|group| group.to_string())
            .collect::<Vec<String>>();

        available_groups.sort();
        available_groups.into_iter().map(QString::from).collect()
    }

    fn updateHostGroups(&mut self, host_id: QString, groups: QStringList) {
        let host_id = host_id.to_string();
        let groups = groups.into_iter().map(|group| group.to_string()).collect::<Vec<String>>();

        let host_settings = self.hosts_config.hosts.get_mut(&host_id).unwrap();
        host_settings.groups = groups;
    }

    /// Returns list of JSON strings representing CustomCommandConfig.
    fn getCustomCommands(&self, host_name: QString) -> QStringList {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).cloned().unwrap_or_default();

        let custom_commands_json = host_settings.effective.custom_commands.iter()
            // `example-command`` used to be in default config, but isn't anymore.
            // TODO: clean up at some point.
            .filter(|command| !(command.name == "example-command" && command.command == "ls -l ~"))
            .map(|command| serde_json::to_string(command).unwrap());

        QStringList::from_iter(custom_commands_json)
    }

    fn updateCustomCommands(&mut self, host_name: QString, custom_commands_json: QStringList) {
        let host_name = host_name.to_string();
        let custom_commands = custom_commands_json.into_iter()
            .map(|json| serde_json::from_str::<configuration::CustomCommandConfig>(&json.to_string()).unwrap())
            .collect::<Vec<configuration::CustomCommandConfig>>();

        let host_settings = self.hosts_config.hosts.get_mut(&host_name).unwrap();
        host_settings.overrides.custom_commands = custom_commands.clone();
        host_settings.effective.custom_commands = custom_commands;
    }

    /// Modules that don't already belong to the group.
    fn getUnselectedMonitorIds(&self, selected_groups: QStringList) -> QStringList {
        let selected_groups = selected_groups.into_iter().map(|group| group.to_string()).collect::<Vec<String>>();

        let all_monitors = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.module_type == ModuleType::Monitor)
            .map(|metadata| metadata.module_spec.id.clone())
            .collect::<Vec<String>>();

        let mut unselected_monitors = all_monitors.into_iter()
            .filter(|monitor| !selected_groups.contains(monitor))
            .collect::<Vec<String>>();

        unselected_monitors.sort();
        QStringList::from_iter(unselected_monitors)
    }

    fn getMonitorDescription(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == ModuleType::Monitor)
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    fn getGroupMonitorIds(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_monitors = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().monitors;

        let mut group_monitors_keys = group_monitors.into_keys().collect::<Vec<String>>();
        group_monitors_keys.sort_by_key(|key| key.to_lowercase());
        QStringList::from_iter(group_monitors_keys)
    }

    fn addGroupMonitor(&mut self, group_name: QString, monitor_name: QString) {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().monitors.insert(monitor_name, Default::default());
    }

    /// Returns a list of JSON serialized `ModuleSetting`. Includes all setting keys.
    fn getGroupMonitorSettings(&self, group_id: QString, module_id: QString) -> QStringList {
        let group_id = group_id.to_string();
        let module_id = module_id.to_string();

        let settings_descriptions = &self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_id && metadata.module_spec.module_type == ModuleType::Monitor)
            .next().unwrap()
            .settings;

        let mut settings_keys = settings_descriptions.keys().collect::<Vec<&String>>();
        settings_keys.sort_by(|&a, &b| a.to_lowercase().cmp(&b.to_lowercase()));

        let group_monitor_settings = self.groups_config
            .groups.get(&group_id).cloned().unwrap_or_default()
            .monitors.get(&module_id).cloned().unwrap_or_default()
            .settings;

        let module_settings = settings_keys.into_iter().map(|setting_key| {
            ModuleSetting {
                key: setting_key.clone(),
                value: group_monitor_settings.get(setting_key).cloned().unwrap_or_default(),
                description: settings_descriptions.get(setting_key).cloned().unwrap_or_default(),
                enabled: group_monitor_settings.get(setting_key).is_some(),
            }
        });

        QStringList::from_iter(module_settings.map(|setting| serde_json::to_string(&setting).unwrap()))
    }

    // Returns a map with module IDs as keys and array of ModuleSettings as values.
    fn getEffectiveMonitorSettings(&self, host_id: QString, grouplist: QStringList) -> QString {
        let host_id = host_id.to_string();
        let grouplist = grouplist.into_iter().map(|group| group.to_string()).collect::<Vec<String>>();

        let mut new_host_config = self.hosts_config.hosts.get(&host_id).cloned().unwrap_or_default();
        new_host_config.groups = grouplist;
        let effective_config = Configuration::get_effective_group_config(&new_host_config, &self.groups_config.groups);

        let modules_settings = effective_config.monitors.iter().map(|(module_id, module_config)| {
            (module_id.clone(), module_config.settings.iter().map(|(key, value)| {
                ModuleSetting {
                    key: key.clone(),
                    value: value.clone(),
                    description: "".into(),
                    enabled: true,
                }
            }).collect::<Vec<ModuleSetting>>())
        }).collect::<HashMap<String, Vec<ModuleSetting>>>();

        QString::from(serde_json::to_string(&modules_settings).unwrap())
    }

    /// Modules that don't already belong to the group.
    fn getUnselectedConnectorIds(&self, selected_groups: QStringList) -> QStringList {
        let selected_groups = selected_groups.into_iter().map(|group| group.to_string()).collect::<Vec<String>>();

        let all_connectors = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.module_type == ModuleType::Connector)
            .map(|metadata| metadata.module_spec.id.clone())
            .collect::<Vec<String>>();

        let mut unselected_connectors = all_connectors.into_iter()
            .filter(|module_id| !selected_groups.contains(module_id))
            .collect::<Vec<String>>();

        unselected_connectors.sort();
        QStringList::from_iter(unselected_connectors)
    }

    fn getConnectorDescription(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == ModuleType::Connector)
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    fn getGroupConnectorIds(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_connectors = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().connectors;

        let mut group_connectors_keys = group_connectors.into_keys().collect::<Vec<String>>();
        group_connectors_keys.sort_by_key(|key| key.to_lowercase());

        QStringList::from_iter(group_connectors_keys)
    }

    fn addGroupConnector(&mut self, group_name: QString, connector_name: QString) {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().connectors.insert(connector_name, Default::default());
    }

    /// Returns a list of JSON serialized `ModuleSetting`. Includes all setting keys.
    fn getGroupConnectorSettings(&self, group_id: QString, module_id: QString) -> QStringList {
        let group_id = group_id.to_string();
        let module_id = module_id.to_string();

        let settings_descriptions = &self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_id && metadata.module_spec.module_type == ModuleType::Connector)
            .next().unwrap()
            .settings;

        let mut settings_keys = settings_descriptions.keys().collect::<Vec<&String>>();
        settings_keys.sort_by(|&a, &b| a.to_lowercase().cmp(&b.to_lowercase()));

        let group_connector_settings = self.groups_config
            .groups.get(&group_id).cloned().unwrap_or_default()
            .connectors.get(&module_id).cloned().unwrap_or_default()
            .settings;

        let module_settings = settings_keys.into_iter().map(|setting_key| {
            ModuleSetting {
                key: setting_key.clone(),
                value: group_connector_settings.get(setting_key).cloned().unwrap_or_default(),
                description: settings_descriptions.get(setting_key).cloned().unwrap_or_default(),
                enabled: group_connector_settings.get(setting_key).is_some(),
            }
        });

        QStringList::from_iter(module_settings.map(|setting| serde_json::to_string(&setting).unwrap()))
    }

    /// Returns a map with module IDs as keys and array of ModuleSettings as values.
    fn getEffectiveConnectorSettings(&self, host_id: QString, grouplist: QStringList) -> QString {
        let host_id = host_id.to_string();
        let grouplist = grouplist.into_iter().map(|group| group.to_string()).collect::<Vec<String>>();

        let mut new_host_config = self.hosts_config.hosts.get(&host_id).cloned().unwrap_or_default();
        new_host_config.groups = grouplist;
        let effective_config = Configuration::get_effective_group_config(&new_host_config, &self.groups_config.groups);

        let modules_settings = effective_config.connectors.iter().map(|(module_id, module_config)| {
            (module_id.clone(), module_config.settings.iter().map(|(key, value)| {
                ModuleSetting {
                    key: key.clone(),
                    value: value.clone(),
                    description: "".into(),
                    enabled: true,
                }
            }).collect::<Vec<ModuleSetting>>())
        }).collect::<HashMap<String, Vec<ModuleSetting>>>();

        QString::from(serde_json::to_string(&modules_settings).unwrap())
    }

    /// Settings should contain JSON serialized hashmaps. Module ID as key and list of ModuleSetting as value.
    fn updateGroupModules(&mut self,
        group_id: QString,
        connector_settings_json: QString,
        monitor_settings_json: QString,
        command_settings_json: QString,
    ) {
        let group_id = group_id.to_string();
        let connector_settings = serde_json::from_str::<HashMap<String, Vec<ModuleSetting>>>(&connector_settings_json.to_string()).unwrap();
        let monitor_settings = serde_json::from_str::<HashMap<String, Vec<ModuleSetting>>>(&monitor_settings_json.to_string()).unwrap();
        let command_settings = serde_json::from_str::<HashMap<String, Vec<ModuleSetting>>>(&command_settings_json.to_string()).unwrap();

        let group = self.groups_config.groups.get_mut(&group_id).unwrap();

        group.connectors.clear();
        for (module_id, settings) in connector_settings {
            let settings = settings.into_iter()
                .filter(|setting| setting.enabled)
                .map(|setting| (setting.key, setting.value))
                .collect::<HashMap<String, String>>();

            group.connectors.entry(module_id).or_insert(Default::default()).settings = settings;
        }

        group.monitors.clear();
        for (module_id, settings) in monitor_settings {
            let settings = settings.into_iter()
                .filter(|setting| setting.enabled)
                .map(|setting| (setting.key, setting.value))
                .collect::<HashMap<String, String>>();

            group.monitors.entry(module_id).or_insert(Default::default()).settings = settings;
        }

        group.commands.clear();
        for (module_id, settings) in command_settings {
            let settings = settings.into_iter()
                .filter(|setting| setting.enabled)
                .map(|setting| (setting.key, setting.value))
                .collect::<HashMap<String, String>>();

            group.commands.entry(module_id).or_insert(Default::default()).settings = settings;
        }
    }

    /// Modules that don't already belong to the group.
    fn getUnselectedCommandIds(&self, selected_groups: QStringList) -> QStringList {
        let selected_groups = selected_groups.into_iter().map(|group| group.to_string()).collect::<Vec<String>>();

        let all_commands = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.module_type == ModuleType::Command)
            .map(|metadata| metadata.module_spec.id.clone())
            .collect::<Vec<String>>();

        let mut unselected_commands = all_commands.into_iter()
            .filter(|module_id| !selected_groups.contains(module_id))
            .collect::<Vec<String>>();

        unselected_commands.sort();
        QStringList::from_iter(unselected_commands)
    }

    fn getCommandDescription(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == ModuleType::Command)
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    fn getGroupCommandIds(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_commands = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().commands;

        let mut group_commands_keys = group_commands.into_keys().collect::<Vec<String>>();
        group_commands_keys.sort_by_key(|key| key.to_lowercase());

        QStringList::from_iter(group_commands_keys)
    }

    fn addGroupCommand(&mut self, group_name: QString, command_name: QString) {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().commands.insert(command_name, Default::default());
    }

    /// Returns a list of JSON serialized `ModuleSetting`. Includes all setting keys.
    fn getGroupCommandSettings(&self, group_id: QString, module_id: QString) -> QStringList {
        let group_id = group_id.to_string();
        let module_id = module_id.to_string();

        let settings_descriptions = &self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_id && metadata.module_spec.module_type == ModuleType::Command)
            .next().unwrap()
            .settings;

        let mut settings_keys = settings_descriptions.keys().collect::<Vec<&String>>();
        settings_keys.sort_by(|&a, &b| a.to_lowercase().cmp(&b.to_lowercase()));

        let group_command_settings = self.groups_config
            .groups.get(&group_id).cloned().unwrap_or_default()
            .commands.get(&module_id).cloned().unwrap_or_default()
            .settings;

        let module_settings = settings_keys.into_iter().map(|setting_key| {
            ModuleSetting {
                key: setting_key.clone(),
                value: group_command_settings.get(setting_key).cloned().unwrap_or_default(),
                description: settings_descriptions.get(setting_key).cloned().unwrap_or_default(),
                enabled: group_command_settings.get(setting_key).is_some(),
            }
        });

        QStringList::from_iter(module_settings.map(|setting| serde_json::to_string(&setting).unwrap()))
    }

    /// Returns a map with module IDs as keys and array of ModuleSettings as values.
    fn getEffectiveCommandSettings(&self, host_id: QString, grouplist: QStringList) -> QString {
        let host_id = host_id.to_string();
        let grouplist = grouplist.into_iter().map(|group| group.to_string()).collect::<Vec<String>>();

        let mut new_host_config = self.hosts_config.hosts.get(&host_id).cloned().unwrap_or_default();
        new_host_config.groups = grouplist;
        let effective_config = Configuration::get_effective_group_config(&new_host_config, &self.groups_config.groups);

        let modules_settings = effective_config.commands.iter().map(|(module_id, module_config)| {
            (module_id.clone(), module_config.settings.iter().map(|(key, value)| {
                ModuleSetting {
                    key: key.clone(),
                    value: value.clone(),
                    description: "".into(),
                    enabled: true,
                }
            }).collect::<Vec<ModuleSetting>>())
        }).collect::<HashMap<String, Vec<ModuleSetting>>>();

        QString::from(serde_json::to_string(&modules_settings).unwrap())
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
            .map(|id| (id, self.getCommandDescription(QString::from(id.clone()))))
            .filter(|(_, description)| !description.is_empty())
            .collect::<Vec<_>>();

        let new_monitors = default_settings.monitors.keys()
            .filter(|id| !group_settings.monitors.contains_key(*id) && !config_helper_data.ignored_monitors.contains(*id))
            // Will return empty description if not found.
            .map(|id| (id, self.getMonitorDescription(QString::from(id.clone()))))
            .filter(|(_, description)| !description.is_empty())
            .collect::<Vec<_>>();

        let new_connectors = default_settings.connectors.keys()
            .filter(|id| !group_settings.connectors.contains_key(*id) && !config_helper_data.ignored_connectors.contains(*id))
            // Will return empty description if not found.
            .map(|id| (id, self.getConnectorDescription(QString::from(id.clone()))))
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

#[derive(Serialize, Deserialize, Default, Clone)]
#[allow(non_snake_case)]
struct ModuleSetting {
    pub key: String,
    pub value: String,
    pub description: String,
    pub enabled: bool,
}