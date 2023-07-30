extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration::{Configuration, Hosts, Groups};


#[derive(QObject, Default)]
pub struct ConfigManagerModel {
    base: qt_base_class!(trait QObject),
    // Returns host settings as JSON string, since it doesn't seem to be possible to return custom QObjects directly.
    get_host_settings: qt_method!(fn(&self, host_name: QString) -> QString),
    get_all_groups: qt_method!(fn(&self) -> QStringList),
    get_group_settings: qt_method!(fn(&self, group_name: QString) -> QString),

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
        let all_groups = self.groups_config.groups.keys().collect::<Vec<&String>>();

        let mut result = QStringList::default();
        for group in all_groups {
            result.push(QString::from(group.clone()));
        }
        result
    }

    pub fn get_group_settings(&self, group_name: QString) -> QString {
        let group_name = group_name.to_string();
        let group_settings = self.groups_config.groups.get(&group_name).unwrap();

        QString::from(serde_json::to_string(&group_settings).unwrap())
    }
}