extern crate qmetaobject;

use qmetaobject::*;

use crate::{
    configuration::Configuration,
    configuration::Hosts,
    configuration::Groups,
    configuration::HostSettings,
    module::Metadata,
};


#[derive(QObject, Default)]
pub struct ConfigManagerModel {
    base: qt_base_class!(trait QObject),

    // Signals
    file_write_error: qt_signal!(config_dir: QString, error_message: QString),

    add_host: qt_method!(fn(&self, host_name: QString)),
    // Returns host settings as JSON string, since it doesn't seem to be possible to return custom QObjects directly.
    get_host_settings: qt_method!(fn(&self, host_name: QString) -> QString),
    set_host_settings: qt_method!(fn(&self, old_host_name: QString, new_host_name: QString, host_settings_json: QString)),
    begin_host_configuration: qt_method!(fn(&self)),
    cancel_host_configuration: qt_method!(fn(&self)),
    end_host_configuration: qt_method!(fn(&self)),

    begin_group_configuration: qt_method!(fn(&self)),
    cancel_group_configuration: qt_method!(fn(&self)),
    end_group_configuration: qt_method!(fn(&self)),

    get_all_groups: qt_method!(fn(&self) -> QStringList),
    add_group: qt_method!(fn(&self, group_name: QString)),
    remove_group: qt_method!(fn(&self, group_name: QString)),
    get_selected_groups: qt_method!(fn(&self, host_name: QString) -> QStringList),
    get_available_groups: qt_method!(fn(&self, host_name: QString) -> QStringList),
    add_host_to_group: qt_method!(fn(&self, host_name: QString, group_name: QString)),
    remove_host_from_group: qt_method!(fn(&self, host_name: QString, group_name: QString)),

    get_all_connectors: qt_method!(fn(&self) -> QStringList),
    get_connector_description: qt_method!(fn(&self, connector_name: QString) -> QString),
    get_group_connectors: qt_method!(fn(&self, group_name: QString) -> QStringList),
    add_group_connector: qt_method!(fn(&self, group_name: QString, connector_name: QString)),
    get_group_connector_settings_keys: qt_method!(fn(&self, group_name: QString, connector_name: QString) -> QStringList),
    get_group_connector_setting: qt_method!(fn(&self, group_name: QString, connector_name: QString, setting_key: QString) -> QString),
    set_group_connector_setting: qt_method!(fn(&self, group_name: QString, connector_name: QString, setting_key: QString, setting_value: QString)),
    remove_group_connector: qt_method!(fn(&self, group_name: QString, connector_name: QString)),

    // NOTE: currently "unset" acts as a special value for indicating if a setting is unset.
    get_all_monitors: qt_method!(fn(&self) -> QStringList),
    get_monitor_description: qt_method!(fn(&self, monitor_name: QString) -> QString),
    get_group_monitors: qt_method!(fn(&self, group_name: QString) -> QStringList),
    add_group_monitor: qt_method!(fn(&self, group_name: QString, monitor_name: QString)),
    remove_group_monitor: qt_method!(fn(&self, group_name: QString, monitor_name: QString)),
    // These 2 are currently not really used.
    get_group_monitor_enabled: qt_method!(fn(&self, group_name: QString, monitor_name: QString) -> QString),
    toggle_group_monitor_enabled: qt_method!(fn(&self, group_name: QString, monitor_name: QString)),
    get_group_monitor_settings_keys: qt_method!(fn(&self, group_name: QString, monitor_name: QString) -> QStringList),
    get_group_monitor_setting: qt_method!(fn(&self, group_name: QString, monitor_name: QString, setting_key: QString) -> QString),
    set_group_monitor_setting: qt_method!(fn(&self, group_name: QString, monitor_name: QString, setting_key: QString, setting_value: QString)),

    get_all_commands: qt_method!(fn(&self) -> QStringList),
    get_command_description: qt_method!(fn(&self, command_name: QString) -> QString),
    get_group_commands: qt_method!(fn(&self, group_name: QString) -> QStringList),
    add_group_command: qt_method!(fn(&self, group_name: QString, command_name: QString)),
    remove_group_command: qt_method!(fn(&self, group_name: QString, command_name: QString)),
    get_group_command_settings_keys: qt_method!(fn(&self, group_name: QString, command_name: QString) -> QStringList),
    get_group_command_setting: qt_method!(fn(&self, group_name: QString, command_name: QString, setting_key: QString) -> QString),
    set_group_command_setting: qt_method!(fn(&self, group_name: QString, command_name: QString, setting_key: QString, setting_value: QString)),

    get_all_module_settings: qt_method!(fn(&self, module_type: QString, module_id: QString) -> QVariantMap),

    config_dir: String,
    hosts_config: Hosts,
    hosts_config_backup: Option<Hosts>,
    groups_config: Groups,
    groups_config_backup: Option<Groups>,
    module_metadatas: Vec<Metadata>,
    // Not yet used.
    _main_config: Configuration,
}

impl ConfigManagerModel {
    pub fn new(config_dir: String,
               main_config: Configuration,
               hosts_config: Hosts,
               groups_config: Groups,
               module_metadatas: Vec<Metadata>) -> Self {
        
        let mut hosts_config = hosts_config;
        // Sort host groups alphabetically.
        for host in hosts_config.hosts.values_mut() {
            host.groups.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        }

        ConfigManagerModel {
            config_dir: config_dir,
            hosts_config: hosts_config,
            groups_config: groups_config,
            _main_config: main_config,
            module_metadatas: module_metadatas,
            ..Default::default()
        }
    }

    pub fn add_host(&mut self, host_name: QString) {
        let host_name = host_name.to_string();
        let config = HostSettings {
            groups: vec![
                String::from("defaults"),
                String::from("linux"),
            ],
            ..Default::default()
        };
        self.hosts_config.hosts.insert(host_name, config);
    }

    pub fn begin_host_configuration(&mut self) {
        self.hosts_config_backup = Some(self.hosts_config.clone());
    }

    pub fn cancel_host_configuration(&mut self) {
        self.hosts_config = self.hosts_config_backup.take().unwrap();
    }

    pub fn end_host_configuration(&mut self) {
        self.hosts_config_backup = None;
        if let Err(error) = Configuration::write_hosts_config(&self.config_dir, &self.hosts_config) {
            self.file_write_error(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
        }
    }

    pub fn begin_group_configuration(&mut self) {
        self.groups_config_backup = Some(self.groups_config.clone());
    }

    pub fn cancel_group_configuration(&mut self) {
        self.groups_config = self.groups_config_backup.take().unwrap();
    }

    pub fn end_group_configuration(&mut self) {
        self.groups_config_backup = None;
        if let Err(error) = Configuration::write_groups_config(&self.config_dir, &self.groups_config) {
            self.file_write_error(QString::from(self.config_dir.clone()), QString::from(error.to_string()));
        }
    }

    pub fn get_host_settings(&self, host_name: QString) -> QString {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).unwrap_or(&Default::default()).clone();

        QString::from(serde_json::to_string(&host_settings).unwrap())
    }

    pub fn set_host_settings(&mut self, old_host_name: QString, new_host_name: QString, host_settings_json: QString) {
        let old_host_name = old_host_name.to_string();
        let new_host_name = new_host_name.to_string();
        let host_settings: HostSettings = serde_json::from_str(&host_settings_json.to_string()).unwrap();

        if old_host_name != new_host_name {
            let host_config = self.hosts_config.hosts.remove(&old_host_name).unwrap();
            self.hosts_config.hosts.insert(new_host_name.clone(), host_config);
        }

        let mut host_config = self.hosts_config.hosts.get_mut(&new_host_name).unwrap();
        host_config.address = host_settings.address;
        host_config.fqdn = host_settings.fqdn;
    }

    pub fn get_all_groups(&self) -> QStringList {
        let mut all_groups = self.groups_config.groups.keys().cloned().collect::<Vec<String>>();
        all_groups.sort();

        let mut result = QStringList::default();
        for group in all_groups {
            result.push(QString::from(group.clone()));
        }
        result
    }

    pub fn add_group(&mut self, group_name: QString) {
        let group_name = group_name.to_string();
        self.groups_config.groups.insert(group_name, Default::default());
    }

    pub fn remove_group(&mut self, group_name: QString) {
        let group_name = group_name.to_string();
        self.groups_config.groups.remove(&group_name);
    }

    pub fn get_selected_groups(&self, host_name: QString) -> QStringList {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).cloned().unwrap_or_default();
        let groups_sorted = {
            let mut groups = host_settings.groups.clone();
            groups.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            groups
        };

        let mut result = QStringList::default();
        for group in groups_sorted {
            result.push(QString::from(group));
        }
        result
    }

    pub fn get_available_groups(&self, host_name: QString) -> QStringList {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).cloned().unwrap_or_default();

        let all_groups = self.groups_config.groups.keys().collect::<Vec<&String>>();
        let available_groups = all_groups.iter()
            .filter(|group| !host_settings.groups.contains(&group))
            .map(|group| group.to_string())
            .collect::<Vec<String>>();

        let mut result = QStringList::default();
        for group in available_groups {
            result.push(QString::from(group));
        }
        result
    }

    pub fn add_host_to_group(&mut self, host_name: QString, group_name: QString) {
        let host_name = host_name.to_string();
        let group_name = group_name.to_string();
        let host_settings = self.hosts_config.hosts.get_mut(&host_name).unwrap();

        host_settings.groups.push(group_name);
    }

    pub fn remove_host_from_group(&mut self, host_name: QString, group_name: QString) {
        let host_name = host_name.to_string();
        let group_name = group_name.to_string();
        let host_settings = self.hosts_config.hosts.get_mut(&host_name).unwrap();

        host_settings.groups.retain(|group| group != &group_name);
    }

    pub fn get_all_monitors(&self) -> QStringList {
        let mut all_monitors = self.module_metadatas.iter().filter(|metadata| metadata.module_spec.module_type == "monitor")
                                                           .map(|metadata| metadata.module_spec.id.clone())
                                                           .collect::<Vec<String>>();
        all_monitors.sort();

        let mut result = QStringList::default();
        for monitor in all_monitors {
            result.push(QString::from(monitor.clone()));
        }
        result
    }

    pub fn get_monitor_description(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == "monitor")
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    pub fn get_group_monitors(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_monitors = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().monitors;

        let mut group_monitors_keys = group_monitors.into_keys().collect::<Vec<String>>();
        group_monitors_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut result = QStringList::default();
        for monitor_key in group_monitors_keys {
            result.push(QString::from(monitor_key));
        }
        result
    }

    pub fn add_group_monitor(&mut self, group_name: QString, monitor_name: QString) {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().monitors.insert(monitor_name, Default::default());
    }

    pub fn remove_group_monitor(&mut self, group_name: QString, monitor_name: QString) {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().monitors.remove(&monitor_name);
    }

    pub fn get_group_monitor_enabled(&self, group_name: QString, monitor_name: QString) -> QString {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();

        self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                          .monitors.get(&monitor_name).cloned().unwrap_or_default()
                          .enabled.unwrap_or(true).to_string().into()
    }

    pub fn toggle_group_monitor_enabled(&mut self, group_name: QString, monitor_name: QString) {
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

    pub fn get_group_monitor_settings_keys(&self, group_name: QString, monitor_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        let group_monitor_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                       .monitors.get(&monitor_name).cloned().unwrap_or_default().settings;

        let mut group_monitor_settings_keys = group_monitor_settings.into_keys().collect::<Vec<String>>();
        group_monitor_settings_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut result = QStringList::default();
        for setting_key in group_monitor_settings_keys {
            result.push(QString::from(setting_key));
        }
        result
    }

    pub fn get_group_monitor_setting(&self, group_name: QString, monitor_name: QString, setting_key: QString) -> QString {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();
        let setting_key = setting_key.to_string();
        let group_monitor_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                       .monitors.get(&monitor_name).cloned().unwrap_or_default().settings;

        QString::from(group_monitor_settings.get(&setting_key).cloned().unwrap_or(String::from("unset")))
    }

    pub fn set_group_monitor_setting(&mut self, group_name: QString, monitor_name: QString, setting_key: QString, setting_value: QString) {
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

    pub fn get_all_connectors(&self) -> QStringList {
        let mut all_connectors = self.module_metadatas.iter().filter(|metadata| metadata.module_spec.module_type == "connector")
                                                             .map(|metadata| metadata.module_spec.id.clone())
                                                             .collect::<Vec<String>>();
        all_connectors.sort();

        let mut result = QStringList::default();
        for connector in all_connectors {
            result.push(QString::from(connector.clone()));
        }
        result
    }

    pub fn get_connector_description(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == "connector")
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    pub fn get_group_connectors(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_connectors = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().connectors;

        let mut group_connectors_keys = group_connectors.into_keys().collect::<Vec<String>>();
        group_connectors_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut result = QStringList::default();
        for connector_key in group_connectors_keys {
            result.push(QString::from(connector_key));
        }
        result
    }

    pub fn add_group_connector(&mut self, group_name: QString, connector_name: QString) {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().connectors.insert(connector_name, Default::default());
    }

    pub fn get_group_connector_settings_keys(&self, group_name: QString, connector_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        let group_connector_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                       .connectors.get(&connector_name).cloned().unwrap_or_default().settings;

        let mut group_connector_settings_keys = group_connector_settings.into_keys().collect::<Vec<String>>();
        group_connector_settings_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut result = QStringList::default();
        for setting_key in group_connector_settings_keys {
            result.push(QString::from(setting_key));
        }
        result
    }

    pub fn get_group_connector_setting(&self, group_name: QString, connector_name: QString, setting_key: QString) -> QString {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        let setting_key = setting_key.to_string();
        let group_connector_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                         .connectors.get(&connector_name).cloned().unwrap_or_default().settings;

        QString::from(group_connector_settings.get(&setting_key).cloned().unwrap_or(String::from("unset")))
    }

    pub fn set_group_connector_setting(&mut self, group_name: QString, connector_name: QString, setting_key: QString, setting_value: QString) {
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

    pub fn remove_group_connector(&mut self, group_name: QString, connector_name: QString) {
        let group_name = group_name.to_string();
        let connector_name = connector_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().connectors.remove(&connector_name);
    }

    pub fn get_all_commands(&self) -> QStringList {
        let mut all_commands = self.module_metadatas.iter().filter(|metadata| metadata.module_spec.module_type == "command")
                                                             .map(|metadata| metadata.module_spec.id.clone())
                                                             .collect::<Vec<String>>();
        all_commands.sort();

        let mut result = QStringList::default();
        for command in all_commands {
            result.push(QString::from(command.clone()));
        }
        result
    }

    pub fn get_command_description(&self, module_name: QString) -> QString {
        let module_name = module_name.to_string();
        let module_description = self.module_metadatas.iter()
            .filter(|metadata| metadata.module_spec.id == module_name && metadata.module_spec.module_type == "command")
            .map(|metadata| metadata.description.clone())
            .next().unwrap_or_default();

        QString::from(module_description)
    }

    pub fn get_group_commands(&self, group_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let group_commands = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default().commands;

        let mut group_commands_keys = group_commands.into_keys().collect::<Vec<String>>();
        group_commands_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut result = QStringList::default();
        for command_key in group_commands_keys {
            result.push(QString::from(command_key));
        }
        result
    }

    pub fn add_group_command(&mut self, group_name: QString, command_name: QString) {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().commands.insert(command_name, Default::default());
    }

    pub fn remove_group_command(&mut self, group_name: QString, command_name: QString) {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        self.groups_config.groups.get_mut(&group_name).unwrap().commands.remove(&command_name);
    }

    pub fn get_group_command_settings_keys(&self, group_name: QString, command_name: QString) -> QStringList {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        let group_command_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                       .commands.get(&command_name).cloned().unwrap_or_default().settings;

        let mut group_command_settings_keys = group_command_settings.into_keys().collect::<Vec<String>>();
        group_command_settings_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut result = QStringList::default();
        for setting_key in group_command_settings_keys {
            result.push(QString::from(setting_key));
        }
        result
    }

    pub fn get_group_command_setting(&self, group_name: QString, command_name: QString, setting_key: QString) -> QString {
        let group_name = group_name.to_string();
        let command_name = command_name.to_string();
        let setting_key = setting_key.to_string();
        let group_command_settings = self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                                                         .commands.get(&command_name).cloned().unwrap_or_default().settings;

        QString::from(group_command_settings.get(&setting_key).cloned().unwrap_or(String::from("unset")))
    }

    pub fn set_group_command_setting(&mut self, group_name: QString, command_name: QString, setting_key: QString, setting_value: QString) {
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

    pub fn get_all_module_settings(&self, module_type: QString, module_id: QString) -> QVariantMap {
        let module_id = module_id.to_string();
        let module_type = module_type.to_string();
        // TODO: Consider version too.
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

}