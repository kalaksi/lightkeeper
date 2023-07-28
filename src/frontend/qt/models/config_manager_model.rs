extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration::{Configuration, Hosts, Groups};


#[derive(QObject, Default)]
pub struct ConfigManagerModel {
    base: qt_base_class!(trait QObject),
    // Returns host settings as JSON string, since it doesn't seem to be possible to return custom QObjects directly.
    get_host_settings: qt_method!(fn(&self, host_name: QString) -> QString),
    get_groups: qt_method!(fn(&self, host_name: QString) -> QStringList),

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

    pub fn get_groups(&self, host_name: QString) -> QStringList {
        let host_groups = self.hosts_config.hosts.get(&host_name.to_string()).unwrap().groups.clone().unwrap_or(Vec::new());
        let mut available_groups = self.groups_config.groups.keys()
                                                            .filter(|name| !host_groups.contains(name))
                                                            .collect::<Vec<&String>>();

        available_groups.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut result = QStringList::default();
        for group in available_groups {
            result.push(QString::from(group.clone()));
        }
        result
    }
}