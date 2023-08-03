extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration::{
    Configuration,
    Hosts,
    Groups,
};


#[derive(QObject, Default)]
pub struct ConfigManagerModel {
    base: qt_base_class!(trait QObject),
    // Returns host settings as JSON string, since it doesn't seem to be possible to return custom QObjects directly.
    get_host_settings: qt_method!(fn(&self, host_name: QString) -> QString),

    get_all_groups: qt_method!(fn(&self) -> QStringList),
    get_selected_groups: qt_method!(fn(&self, host_name: QString) -> QStringList),
    get_available_groups: qt_method!(fn(&self, host_name: QString) -> QStringList),
    add_host_to_group: qt_method!(fn(&self, host_name: QString, group_name: QString)),
    remove_host_from_group: qt_method!(fn(&self, host_name: QString, group_name: QString)),

    get_group_monitors: qt_method!(fn(&self, group_name: QString) -> QStringList),
    get_group_monitor_enabled: qt_method!(fn(&self, group_name: QString, monitor_name: QString) -> QString),
    get_group_monitor_settings_keys: qt_method!(fn(&self, group_name: QString, monitor_name: QString) -> QStringList),
    get_group_monitor_setting: qt_method!(fn(&self, group_name: QString, monitor_name: QString, setting_key: QString) -> QString),

    get_group_connectors: qt_method!(fn(&self, group_name: QString) -> QStringList),
    get_group_connector_settings_keys: qt_method!(fn(&self, group_name: QString, connector_name: QString) -> QStringList),
    get_group_connector_setting: qt_method!(fn(&self, group_name: QString, connector_name: QString, setting_key: QString) -> QString),

    hosts_config: Hosts,
    groups_config: Groups,
    // Not yet used.
    _main_config: Configuration,
}

impl ConfigManagerModel {
    pub fn new(main_config: Configuration, mut hosts_config: Hosts, groups_config: Groups) -> ConfigManagerModel {
        // Sort host groups alphabetically.
        for host in hosts_config.hosts.values_mut() {
            if let Some(ref mut groups) = host.groups {
                groups.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            }
        }

        ConfigManagerModel {
            hosts_config: hosts_config,
            groups_config: groups_config,
            _main_config: main_config,
            ..Default::default()
        }
    }

    pub fn get_host_settings(&self, host_name: QString) -> QString {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).unwrap();

        QString::from(serde_json::to_string(&host_settings).unwrap())
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

    pub fn get_selected_groups(&self, host_name: QString) -> QStringList {
        let host_name = host_name.to_string();
        let host_settings = self.hosts_config.hosts.get(&host_name).cloned().unwrap_or_default();
        let groups_sorted = match host_settings.groups {
            Some(groups) => {
                let mut groups_sorted = groups.clone();
                groups_sorted.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                groups_sorted
            },
            None => Vec::new(),
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
        let available_groups = all_groups.iter().filter(|group| !host_settings.groups.as_ref().unwrap().contains(&group)).cloned()
                                         .collect::<Vec<&String>>();

        let mut result = QStringList::default();
        for group in available_groups {
            result.push(QString::from(group.to_owned()));
        }
        result
    }

    pub fn add_host_to_group(&mut self, host_name: QString, group_name: QString) {
        let host_name = host_name.to_string();
        let group_name = group_name.to_string();
        let host_settings = self.hosts_config.hosts.get_mut(&host_name).unwrap();

        if let Some(ref mut groups) = host_settings.groups {
            groups.push(group_name);
        } else {
            host_settings.groups = Some(vec![group_name]);
        }
    }

    pub fn remove_host_from_group(&mut self, host_name: QString, group_name: QString) {
        let host_name = host_name.to_string();
        let group_name = group_name.to_string();
        let host_settings = self.hosts_config.hosts.get_mut(&host_name).unwrap();

        if let Some(ref mut groups) = host_settings.groups {
            groups.retain(|group| group != &group_name);
        }
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

    pub fn get_group_monitor_enabled(&self, group_name: QString, monitor_name: QString) -> QString {
        let group_name = group_name.to_string();
        let monitor_name = monitor_name.to_string();

        self.groups_config.groups.get(&group_name).cloned().unwrap_or_default()
                          .monitors.get(&monitor_name).cloned().unwrap_or_default()
                          .enabled.unwrap_or(true).to_string().into()
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

        QString::from(group_monitor_settings.get(&setting_key).cloned().unwrap_or_default())
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

        QString::from(group_connector_settings.get(&setting_key).cloned().unwrap_or_default())
    }
}