use serde_derive::{ Serialize, Deserialize };
use serde_yaml;
use std::{ fs, collections::HashMap };
use crate::utils::enums::HostStatus;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub general: General,
    pub defaults: Defaults,
    pub display_options: DisplayOptions,
    pub hosts: HashMap<String, Host>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct General {
    pub default_host_status: HostStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayOptions {
    pub excluded_monitors: Vec<String>,
    pub group_multivalue: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    pub monitors: HashMap<String, HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub address: Option<String>,
    pub fqdn: Option<String>,
    #[serde(default)]
    pub monitors: HashMap<String, MonitorConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorConfig {
    pub version: String,
    pub is_critical: Option<bool>,
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl Configuration {
    pub fn read(filename: &String) -> Result<Configuration, String> {
        let contents = fs::read_to_string(filename).map_err(|e| e.to_string())?;
        let mut config = serde_yaml::from_str::<Configuration>(contents.as_str()).map_err(|e| e.to_string())?;

        // Apply defaults.
        for (host_name, host_config) in &mut config.hosts.iter_mut() {
            for (monitor_name, monitor_data) in &mut host_config.monitors.iter_mut() {
                if let Some(defaults) = config.defaults.monitors.get(monitor_name) {
                    let mut unified = defaults.clone();
                    unified.extend(monitor_data.settings.clone());
                    monitor_data.settings = unified;
                }
            }
        }

        Ok(config)
    }
}

