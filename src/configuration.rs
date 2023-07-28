use serde_derive::{ Serialize, Deserialize };
use serde_yaml;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{ fs, io, collections::HashMap };
use crate::host::HostSetting;
use crate::file_handler;

const MAIN_CONFIG_FILE: &str = "config.yml";
const HOSTS_FILE: &str = "hosts.yml";
const GROUPS_FILE: &str = "groups.yml";


#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub preferences: Preferences,
    pub display_options: DisplayOptions,
    pub cache_settings: CacheSettings,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Groups {
    pub groups: HashMap<String, HostSettings>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Hosts {
    pub hosts: HashMap<String, HostSettings>,
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
    pub hide_info_notifications: bool,
    pub categories: HashMap<String, Category>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct CacheSettings {
    /// Enable cache. Set false to disable completely and make sure cache file is empty.
    /// Otherwise, cache file is maintained even if it's not used at that moment. This setting will make sure it's not used at all.
    pub enable_cache: bool,
    /// Cache provides an initial value before receiving the up-to-date value.
    pub provide_initial_value: bool,
    /// How long entries in cache are considered valid.
    pub initial_value_time_to_live: u64,
    /// If enabled, value is returned only from cache if it is available.
    pub prefer_cache: bool,
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
pub struct HostSettings {
    pub groups: Option<Vec<String>>,
    #[serde(default = "HostSettings::default_address")]
    pub address: String,
    #[serde(default = "HostSettings::default_fqdn")]
    pub fqdn: String,
    #[serde(default)]
    pub monitors: HashMap<String, MonitorConfig>,
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>,
    #[serde(default)]
    pub connectors: HashMap<String, ConnectorConfig>,
    #[serde(default)]
    pub settings: Option<Vec<HostSetting>>,
}

impl HostSettings {
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

impl Default for MonitorConfig {
    fn default() -> Self {
        MonitorConfig {
            version: MonitorConfig::default_version(),
            is_critical: None,
            settings: HashMap::new(),
        }
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

impl Default for CommandConfig {
    fn default() -> Self {
        CommandConfig {
            version: CommandConfig::default_version(),
            settings: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConnectorConfig {
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl Configuration {
    pub fn read(config_dir: &String) -> io::Result<(Configuration, Hosts)> {
        let config_dir = if config_dir.is_empty() {
            file_handler::get_config_dir().unwrap()
        }
        else {
            Path::new(config_dir).to_path_buf()
        };

        let main_config_file_path = config_dir.join(MAIN_CONFIG_FILE);
        let hosts_file_path = config_dir.join(HOSTS_FILE);
        let groups_file_path = config_dir.join(GROUPS_FILE);

        // If main configuration is missing, this is probably the first run, so create initial configurations.
        if let Err(_) = fs::metadata(&main_config_file_path) {
            Self::write_initial_config(config_dir)?;
        }

        log::info!("Reading main configuration from {}", main_config_file_path.display());
        let config_contents = fs::read_to_string(main_config_file_path)?;

        let main_config = serde_yaml::from_str::<Configuration>(config_contents.as_str())
                                     .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        log::info!("Reading host configuration from {}", hosts_file_path.display());
        let hosts_contents = fs::read_to_string(hosts_file_path)?;
        let mut hosts = serde_yaml::from_str::<Hosts>(hosts_contents.as_str())
                                   .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        log::info!("Reading group configuration from {}", groups_file_path.display());
        let groups_contents = fs::read_to_string(groups_file_path)?;
        let all_groups = serde_yaml::from_str::<Groups>(groups_contents.as_str())
                                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        // Check there are no invalid group references.
        let invalid_groups = hosts.hosts.values()
            .filter(|host_config| host_config.groups.is_some())
            .flat_map(|host_config| host_config.groups.clone().unwrap_or_else(|| Vec::new()))
            .filter(|group_id| !all_groups.groups.contains_key(group_id))
            .collect::<Vec<String>>();

        if !invalid_groups.is_empty() {
            let error_message = format!("Invalid group references: {}", invalid_groups.join(", "));
            return Err(io::Error::new(io::ErrorKind::Other, error_message));
        }

        for (_, host_config) in hosts.hosts.iter_mut() {
            if let Some(host_group_ids) = host_config.groups.clone() {
                for group_id in host_group_ids.iter() {
                    let group_config = all_groups.groups.get(group_id).unwrap();

                    // Host settings are not merged.
                    if group_config.settings.is_some() {
                        host_config.settings = group_config.settings.clone();
                    }

                    // Merge groups.
                    group_config.monitors.iter().for_each(|(monitor_id, new_config)| {
                        let mut merged_config = host_config.monitors.get(monitor_id).cloned().unwrap_or(MonitorConfig::default());
                        merged_config.settings.extend(new_config.settings.clone());
                        merged_config.is_critical = new_config.is_critical;
                        host_config.monitors.insert(monitor_id.clone(), merged_config);
                    });

                    group_config.commands.iter().for_each(|(command_id, new_config)| {
                        let mut merged_config = host_config.commands.get(command_id).cloned().unwrap_or(CommandConfig::default());
                        merged_config.settings.extend(new_config.settings.clone());
                        merged_config.version = new_config.version.clone();
                        host_config.commands.insert(command_id.clone(), merged_config);
                    });

                    group_config.connectors.iter().for_each(|(connector_id, new_config)| {
                        let mut merged_config = host_config.connectors.get(connector_id).cloned().unwrap_or(ConnectorConfig::default());
                        merged_config.settings.extend(new_config.settings.clone());
                        host_config.connectors.insert(connector_id.clone(), merged_config);
                    });
                }
            }
        }

        Ok((main_config, hosts))
    }

    pub fn write_initial_config(config_dir: PathBuf) -> io::Result<()> {
        let default_main_config = include_str!("../config.example.yml");
        let default_hosts_config = include_str!("../hosts.example.yml");
        let default_groups_config = include_str!("../groups.example.yml");

        let main_config_file_path = config_dir.join(MAIN_CONFIG_FILE);
        let hosts_file_path = config_dir.join(HOSTS_FILE);
        let groups_file_path = config_dir.join(GROUPS_FILE);

        fs::create_dir_all(&config_dir)?;

        let main_config_file = fs::OpenOptions::new().write(true).create_new(true).open(main_config_file_path.clone());
        match main_config_file {
            Ok(mut file) => {
                if let Err(error) = file.write_all(default_main_config.as_bytes()) {
                    let message = format!("Failed to write main configuration file {}: {}", main_config_file_path.to_string_lossy(), error);
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Created new main configuration file {}", main_config_file_path.to_string_lossy());
                }
            },
            Err(error) => {
                let message = format!("Failed to create main configuration file {}: {}", main_config_file_path.to_string_lossy(), error);
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        let hosts_config_file = fs::OpenOptions::new().write(true).create_new(true).open(hosts_file_path.clone());
        match hosts_config_file {
            Ok(mut file) => {
                if let Err(error) = file.write_all(default_hosts_config.as_bytes()) {
                    let message = format!("Failed to write host configuration file {}: {}", hosts_file_path.to_string_lossy(), error);
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Created new host configuration file {}", hosts_file_path.to_string_lossy());
                }
            },
            Err(error) => {
                let message = format!("Failed to create host configuration file {}: {}", hosts_file_path.to_string_lossy(), error);
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        let groups_config_file = fs::OpenOptions::new().write(true).create_new(true).open(groups_file_path.clone());
        match groups_config_file {
            Ok(mut file) => {
                if let Err(error) = file.write_all(default_groups_config.as_bytes()) {
                    let message = format!("Failed to write group configuration file {}: {}", groups_file_path.to_string_lossy(), error);
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Created new group configuration file {}", groups_file_path.to_string_lossy());
                }
            },
            Err(error) => {
                let message = format!("Failed to create group configuration file {}: {}", groups_file_path.to_string_lossy(), error);
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        Ok(())
    }

    /// Writes the hosts.yml configuration file.
    pub fn write_hosts_config(config_dir: PathBuf, hosts: &Hosts) -> io::Result<()> {
        let hosts_file_path = config_dir.join(HOSTS_FILE);

        let hosts_config_file = fs::OpenOptions::new().write(true).truncate(true).open(hosts_file_path.clone());
        match hosts_config_file {
            Ok(mut file) => {
                let hosts_config = serde_yaml::to_string(hosts).unwrap();
                if let Err(error) = file.write_all(hosts_config.as_bytes()) {
                    let message = format!("Failed to write host configuration file {}: {}", hosts_file_path.to_string_lossy(), error);
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Updated host configuration file {}", hosts_file_path.to_string_lossy());
                }
            },
            Err(error) => {
                let message = format!("Failed to open host configuration file {}: {}", hosts_file_path.to_string_lossy(), error);
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        Ok(())
    }
}
