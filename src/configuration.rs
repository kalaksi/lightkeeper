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
    pub group_multivalue: bool,
    pub category_order: Vec<String>,
    pub command_order: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    pub connectors: HashMap<String, HashMap<String, String>>,
    pub monitors: HashMap<String, HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    #[serde(default = "Host::default_address")]
    pub address: String,
    #[serde(default = "Host::default_fqdn")]
    pub fqdn: String,
    #[serde(default)]
    pub monitors: HashMap<String, MonitorConfig>,
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>,
    #[serde(default)]
    pub connectors: HashMap<String, ConnectorConfig>,
}

impl Host {
    pub fn default_address() -> String {
        String::from("0.0.0.0")
    }

    pub fn default_fqdn() -> String {
        String::from("")
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorConfig {
    pub version: String,
    pub is_critical: Option<bool>,
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandConfig {
    pub version: String,
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorConfig {
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl Configuration {
    pub fn read(filename: &String) -> Result<Configuration, String> {
        let contents = fs::read_to_string(filename).map_err(|e| e.to_string())?;
        let mut config = serde_yaml::from_str::<Configuration>(contents.as_str()).map_err(|e| e.to_string())?;

        // Apply defaults.
        for (_, host_config) in &mut config.hosts.iter_mut() {
            for (monitor_id, monitor_config) in &mut host_config.monitors.iter_mut() {
                if let Some(defaults) = config.defaults.monitors.get(monitor_id) {
                    let mut unified = defaults.clone();
                    unified.extend(monitor_config.settings.clone());
                    monitor_config.settings = unified;
                }
            }

            for (connector_id, connector_config) in config.defaults.connectors.iter() {
                host_config.connectors.insert(connector_id.clone(),
                                              ConnectorConfig { settings: connector_config.clone() });
            }
        }

        Ok(config)
    }
}

