use serde_derive::{ Serialize, Deserialize };
use serde_yaml;
use std::{ fs, io, collections::HashMap };
use crate::host::HostSetting;


#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub preferences: Preferences,
    pub display_options: DisplayOptions,
    pub cache_settings: CacheSettings,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Templates {
    pub templates: HashMap<String, Host>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hosts {
    pub hosts: HashMap<String, Host>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub refresh_hosts_on_start: bool,
    pub use_remote_editor: bool,
    pub sudo_remote_editor: bool,
    // TODO: check for valid command.
    pub remote_text_editor: String,
    // TODO: check for valid path.
    pub text_editor: String,
    pub terminal: String,
    pub terminal_args: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct DisplayOptions {
    pub group_multivalue: bool,
    pub categories: HashMap<String, Category>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct CacheSettings {
    /// Cache provides an initial value before receiving the up-to-date value.
    pub provide_initial_value: bool,
    /// How long entries in cache are considered valid.
    pub initial_value_time_to_live: u64,
    /// If enabled, value is returned only from cache if it is available.
    pub use_cache: bool,
    /// How long entries in cache are considered valid.
    pub time_to_live: u64,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub priority: u16,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub command_order: Option<Vec<String>>,
    pub monitor_order: Option<Vec<String>>,
    pub collapsible_commands: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub templates: Option<Vec<String>>,
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
    #[serde(default)]
    pub settings: Vec<HostSetting>,
}

impl Host {
    pub fn default_address() -> String {
        String::from("0.0.0.0")
    }

    pub fn default_fqdn() -> String {
        String::from("")
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MonitorConfig {
    #[serde(default = "MonitorConfig::default_version")]
    pub version: String,
    pub is_critical: Option<bool>,
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl MonitorConfig {
    pub fn default_version() -> String {
        String::from("latest")
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CommandConfig {
    #[serde(default = "CommandConfig::default_version")]
    pub version: String,
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl CommandConfig {
    pub fn default_version() -> String {
        String::from("latest")
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConnectorConfig {
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl Configuration {
    pub fn read(config_file_name: &String, hosts_file_name: &String, templates_file_name: &String) -> io::Result<(Configuration, Hosts)> {
        log::debug!("Reading general configuration from {}", config_file_name);
        let config_contents = fs::read_to_string(config_file_name)?;
        let main_config = serde_yaml::from_str::<Configuration>(config_contents.as_str())
                                     .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        log::debug!("Reading host configuration from {}", hosts_file_name);
        let hosts_contents = fs::read_to_string(hosts_file_name)?;
        let mut hosts = serde_yaml::from_str::<Hosts>(hosts_contents.as_str())
                                   .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        log::debug!("Reading template configuration from {}", templates_file_name);
        let templates_contents = fs::read_to_string(templates_file_name)?;
        let all_templates = serde_yaml::from_str::<Templates>(templates_contents.as_str())
                                   .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        // Apply templates.
        for (_, host_config) in hosts.hosts.iter_mut() {
            if let Some(host_templates) = host_config.templates.clone() {
                for template_id in host_templates.iter().rev() {
                    if let Some(template_config) = all_templates.templates.get(template_id) {
                        let mut monitors = template_config.monitors.clone();
                        monitors.extend(host_config.monitors.to_owned());
                        host_config.monitors = monitors;

                        let mut commands = template_config.commands.clone();
                        commands.extend(host_config.commands.to_owned());
                        host_config.commands = commands;

                        let mut connectors = template_config.connectors.clone();
                        connectors.extend(host_config.connectors.to_owned());
                        host_config.connectors = connectors;

                        let mut settings = template_config.settings.clone();
                        settings.extend(host_config.settings.to_owned());
                        host_config.settings = settings;
                    }
                    else {
                        panic!("No such template");
                    }
                }
            }
        }

        Ok((main_config, hosts))
    }
}