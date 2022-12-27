use serde_derive::{ Serialize, Deserialize };
use serde_yaml;
use std::{ fs, io, collections::HashMap };
use crate::utils::enums::HostStatus;


#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub preferences: Preferences,
    pub general: General,
    pub defaults: Defaults,
    pub display_options: DisplayOptions,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hosts {
    pub hosts: HashMap<String, Host>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub use_remote_editor: bool,
    pub sudo_remote_editor: bool,
    // TODO: check for valid command.
    pub remote_text_editor: String,
    // TODO: check for valid path.
    pub text_editor: String,
    pub terminal: String,
    pub terminal_args: Vec<String>,
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
    pub fn read(config_file_name: &String, hosts_file_name: &String) -> io::Result<(Configuration, Hosts)> {
        let config_contents = fs::read_to_string(config_file_name)?;
        let hosts_contents = fs::read_to_string(hosts_file_name)?;
        let mut main_config = serde_yaml::from_str::<Configuration>(config_contents.as_str())
                                         .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;
        let mut hosts = serde_yaml::from_str::<Hosts>(hosts_contents.as_str())
                                   .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        // Apply defaults.
        for (_, host_config) in hosts.hosts.iter_mut() {
            for (monitor_id, monitor_config) in &mut host_config.monitors.iter_mut() {
                if let Some(defaults) = main_config.defaults.monitors.get(monitor_id) {
                    let mut unified = defaults.clone();
                    unified.extend(monitor_config.settings.clone());
                    monitor_config.settings = unified;
                }
            }

            for (connector_id, connector_config) in main_config.defaults.connectors.iter() {
                host_config.connectors.insert(connector_id.clone(),
                                              ConnectorConfig { settings: connector_config.clone() });
            }
        }

        Ok((main_config, hosts))
    }
}

