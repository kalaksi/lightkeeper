/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs, io};

use serde_derive::{Deserialize, Serialize};
use serde_yaml;

use crate::file_handler;
use crate::host::HostSetting;

const MAIN_CONFIG_FILE: &str = "config.yml";
const HOSTS_FILE: &str = "hosts.yml";
const GROUPS_FILE: &str = "groups.yml";
const DEFAULT_GROUPS_CONFIG: &str = include_str!("../groups.example.yml");
const DEFAULT_MAIN_CONFIG: &str = include_str!("../config.example.yml");
const DEFAULT_HOSTS_CONFIG: &str = include_str!("../hosts.example.yml");
pub const INTERNAL: &str = "internal";
pub const CURRENT_SCHEMA_VERSION: u16 = 2;

#[derive(Serialize, Debug, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default)]
    pub display_options: DisplayOptions,
    // Obsolete field:
    #[serde(default, skip_serializing_if = "Configuration::always")]
    pub cache_settings: Option<serde_yaml::Value>,
    #[serde(default)]
    pub schema_version: Option<u16>,
}

impl Default for Preferences {
    fn default() -> Self {
        let default_main_config = get_default_main_config();
        default_main_config.preferences
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Groups {
    pub groups: BTreeMap<String, ConfigGroup>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Hosts {
    pub hosts: BTreeMap<String, HostSettings>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub certificate_monitors: Vec<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    #[serde(default)]
    pub use_sandbox_mode: bool,
    pub refresh_hosts_on_start: bool,
    pub use_remote_editor: bool,
    pub sudo_remote_editor: bool,
    pub remote_text_editor: String,
    /// Command to run when launching a text editor. "internal" is a special value that uses the internal editor.
    pub text_editor: String,
    /// Command to run when launching a terminal. "internal" is a special value that uses the internal terminal.
    pub terminal: String,
    pub terminal_args: Vec<String>,
    #[serde(default = "DisplayOptions::default_to_true")]
    pub close_to_tray: bool,
    #[serde(default = "DisplayOptions::default_to_true")]
    pub show_monitor_notifications: bool,
    #[serde(default)]
    pub show_charts: bool,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DisplayOptions {
    #[serde(default)]
    pub qtquick_style: String,
    #[serde(default)]
    pub hide_info_notifications: bool,
    // Currently not writing everything in the config file since they are not user-configurable.
    #[serde(default, skip_serializing_if = "Configuration::always")]
    // TODO: could be a list and 'priority' removed.
    pub categories: HashMap<String, Category>,
    #[serde(default = "DisplayOptions::default_to_true")]
    pub show_status_bar: bool,
    #[serde(default, skip_serializing_if = "Configuration::always")]
    pub chart_categories: Vec<ChartCategory>,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        let default_main_config = get_default_main_config();
        default_main_config.display_options
    }
}

impl DisplayOptions {
    pub fn default_to_true() -> bool {
        true
    }
}

#[derive(Serialize, Debug, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub priority: u16,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub command_order: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub monitor_order: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub collapsible_commands: Option<Vec<String>>,
}

#[derive(Serialize, Debug, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChartCategory {
    pub name: String,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub monitors: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HostSettings {
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default = "HostSettings::default_address", skip_serializing_if = "HostSettings::is_default_address")]
    pub address: String,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub fqdn: String,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub overrides: ConfigGroup,
    /// Effective configuration after merging everything. Will not be stored in config file, but is available in runtime.
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub effective: ConfigGroup,

    /// Deprecated.
    #[serde(default, skip_serializing_if = "Configuration::always")]
    pub monitors: BTreeMap<String, MonitorConfig>,
    /// Deprecated.
    #[serde(default, skip_serializing_if = "Configuration::always")]
    pub commands: BTreeMap<String, CommandConfig>,
    /// Deprecated.
    #[serde(default, skip_serializing_if = "Configuration::always")]
    pub connectors: BTreeMap<String, ConnectorConfig>,
    /// Deprecated.
    #[serde(default, skip_serializing_if = "Configuration::always")]
    pub settings: Vec<HostSetting>,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct ConfigGroup {
    // Hashmap keys are always names/ids.
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub monitors: BTreeMap<String, MonitorConfig>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub commands: BTreeMap<String, CommandConfig>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub custom_commands: Vec<CustomCommandConfig>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub connectors: BTreeMap<String, ConnectorConfig>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub host_settings: Vec<HostSetting>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub config_helper: ConfigHelperData,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct ConfigHelperData {
    pub ignored_commands: Vec<String>,
    pub ignored_monitors: Vec<String>,
    pub ignored_connectors: Vec<String>,
}

impl HostSettings {
    pub fn default_address() -> String {
        String::from("0.0.0.0")
    }

    pub fn is_default_address(address: &String) -> bool {
        address == "0.0.0.0"
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MonitorConfig {
    #[serde(default = "MonitorConfig::default_version", skip_serializing_if = "Configuration::version_is_latest")]
    pub version: String,
    #[serde(default = "MonitorConfig::default_enabled", skip_serializing_if = "MonitorConfig::is_enabled")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Configuration::is_default")]
    pub is_critical: Option<bool>,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub settings: HashMap<String, String>,
}

impl MonitorConfig {
    pub fn default_version() -> String {
        String::from("latest")
    }

    pub fn default_enabled() -> Option<bool> {
        Some(true)
    }

    pub fn is_enabled(enabled: &Option<bool>) -> bool {
        (*enabled).unwrap_or(true)
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        MonitorConfig {
            version: MonitorConfig::default_version(),
            enabled: MonitorConfig::default_enabled(),
            is_critical: None,
            settings: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CommandConfig {
    #[serde(default = "CommandConfig::default_version", skip_serializing_if = "Configuration::version_is_latest")]
    pub version: String,
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
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

#[derive(Default, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CustomCommandConfig {
    pub name: String,
    pub description: String,
    pub command: String,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ConnectorConfig {
    #[serde(default, skip_serializing_if = "Configuration::is_default")]
    pub settings: HashMap<String, String>,
}

impl Configuration {
    #![cfg_attr(any(), rustfmt::skip)]
    pub fn read(config_dir: &str) -> io::Result<(Configuration, Hosts, Groups)> {
        let config_dir = if config_dir.is_empty() {
            file_handler::get_config_dir()
        }
        else {
            Path::new(config_dir).to_path_buf()
        };

        let main_config_file_path = config_dir.join(MAIN_CONFIG_FILE);
        let hosts_file_path = config_dir.join(HOSTS_FILE);
        let groups_file_path = config_dir.join(GROUPS_FILE);
        let old_templates_file_path = config_dir.join("templates.yml");

        // If main configuration is missing, this is probably the first run, so create initial configurations.
        if fs::metadata(&main_config_file_path).is_err() {
            Self::write_initial_config(&config_dir)?;
        }
        else if fs::metadata(config_dir.join("templates.yml")).is_ok() {
            log::warn!("Old templates.yml configuration file found. Renaming old configuration files and reinitializing.");

            // This is the old groups.yml file. Rename old files with .old suffix and do a new init.
            fs::rename(&main_config_file_path, config_dir.join(format!("{}.old", MAIN_CONFIG_FILE)))?;
            fs::rename(&hosts_file_path, config_dir.join(format!("{}.old", HOSTS_FILE)))?;
            fs::rename(old_templates_file_path, config_dir.join("templates.yml.old"))?;

            Self::write_initial_config(&config_dir)?;
        }

        // Make sure directory is protected from reading by others.
        if let Err(error) = fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700)) {
            log::error!("Error while setting config directory permissions: {}", error);
        }

        log::info!("Reading main configuration from {}", main_config_file_path.display());
        let config_contents = fs::read_to_string(main_config_file_path)?;

        let mut main_config = serde_yaml::from_str::<Configuration>(config_contents.as_str())
                                         .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

        // Display options are currently defined in the app's defaults and not really user-configurable.
        let mut actual_display_options = get_default_main_config().display_options;

        // Exceptions. Allow some to be configurable.
        actual_display_options.show_status_bar = main_config.display_options.show_status_bar;
        main_config.display_options = actual_display_options;

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
            .flat_map(|host_config| host_config.groups.clone())
            .filter(|group_id| !all_groups.groups.contains_key(group_id))
            .collect::<Vec<String>>();

        if !invalid_groups.is_empty() {
            let error_message = format!("Invalid group references: {}", invalid_groups.join(", "));
            return Err(io::Error::new(io::ErrorKind::Other, error_message));
        }

        // Merge config groups to form the final, effective config.
        for (_, host_config) in hosts.hosts.iter_mut() {
            host_config.effective = Self::get_effective_group_config(host_config, &all_groups.groups);

            // Old, deprecated host overrides.
            let old_overrides = ConfigGroup {
                commands: host_config.commands.clone(),
                monitors: host_config.monitors.clone(),
                connectors: host_config.connectors.clone(),
                custom_commands: Vec::new(),
                host_settings: host_config.settings.clone(),
                config_helper: Default::default(),
            };

            // New host overrides.
            host_config.overrides = Self::merge_group_config(&old_overrides, &host_config.overrides);

            // Clear old, deprecated settings.
            host_config.commands = BTreeMap::new();
            host_config.monitors = BTreeMap::new();
            host_config.connectors = BTreeMap::new();
            host_config.settings = Vec::new();
        }

        Ok((main_config, hosts, all_groups))
    }

    /// Merge config groups to form the final, effective config.
    pub fn get_effective_group_config(host_config: &HostSettings, all_groups: &BTreeMap<String, ConfigGroup>) -> ConfigGroup {
        let mut effective_config = ConfigGroup::default();

        for group_id in host_config.groups.iter() {
            match all_groups.get(group_id) {
                Some(group_config) => {
                    effective_config = Self::merge_group_config(&effective_config, group_config);
                }
                None => {
                    log::error!("Group {} not found when creating effective configuration", group_id);
                }
            }
        }

        // Old, deprecated host overrides.
        let old_overrides = ConfigGroup {
            commands: host_config.commands.clone(),
            monitors: host_config.monitors.clone(),
            connectors: host_config.connectors.clone(),
            custom_commands: Vec::new(),
            host_settings: host_config.settings.clone(),
            config_helper: Default::default(),
        };

        let all_overrides = Self::merge_group_config(&old_overrides, &host_config.overrides);

        effective_config = Self::merge_group_config(&effective_config, &all_overrides);
        effective_config
    }

    /// Merges configuration groups, second parameter will overwrite conflicting contents from first.
    pub fn merge_group_config(first_config: &ConfigGroup, second_config: &ConfigGroup) -> ConfigGroup {
        let mut result = first_config.clone();

        second_config.monitors.iter().for_each(|(monitor_id, new_config)| {
            let mut merged_config = first_config.monitors.get(monitor_id).cloned().unwrap_or_default();
            merged_config.settings.extend(new_config.settings.clone());
            merged_config.enabled = new_config.enabled.clone();
            merged_config.is_critical = new_config.is_critical;
            result.monitors.insert(monitor_id.clone(), merged_config);
        });

        second_config.commands.iter().for_each(|(command_id, new_config)| {
            let mut merged_config = first_config.commands.get(command_id).cloned().unwrap_or_default();
            merged_config.settings.extend(new_config.settings.clone());
            merged_config.version = new_config.version.clone();
            result.commands.insert(command_id.clone(), merged_config);
        });

        second_config.connectors.iter().for_each(|(connector_id, new_config)| {
            let mut merged_config = first_config.connectors.get(connector_id).cloned().unwrap_or_default();
            merged_config.settings.extend(new_config.settings.clone());
            result.connectors.insert(connector_id.clone(), merged_config);
        });

        second_config.custom_commands.iter().for_each(|new_config| {
            result.custom_commands.push(new_config.clone());
        });

        if second_config.host_settings.len() > 0 {
            result.host_settings = second_config.host_settings.clone();
        }

        result
    }

    pub fn write_initial_config(config_dir: &PathBuf) -> io::Result<()> {
        let main_config_file_path = config_dir.join(MAIN_CONFIG_FILE);
        let hosts_file_path = config_dir.join(HOSTS_FILE);
        let groups_file_path = config_dir.join(GROUPS_FILE);

        fs::create_dir_all(config_dir)?;

        let main_config_file = fs::OpenOptions::new().write(true).create_new(true).open(main_config_file_path.clone());
        match main_config_file {
            Ok(mut file) => {
                if let Err(error) = file.write_all(DEFAULT_MAIN_CONFIG.as_bytes()) {
                    let message = format!(
                        "Failed to write main configuration file {}: {}",
                        main_config_file_path.to_string_lossy(),
                        error
                    );
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Created new main configuration file {}", main_config_file_path.to_string_lossy());
                }
            }
            Err(error) => {
                let message = format!(
                    "Failed to create main configuration file {}: {}",
                    main_config_file_path.to_string_lossy(),
                    error
                );
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        let hosts_config_file = fs::OpenOptions::new().write(true).create_new(true).open(hosts_file_path.clone());
        match hosts_config_file {
            Ok(mut file) => {
                if let Err(error) = file.write_all(DEFAULT_HOSTS_CONFIG.as_bytes()) {
                    let message = format!("Failed to write host configuration file {}: {}", hosts_file_path.to_string_lossy(), error);
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Created new host configuration file {}", hosts_file_path.to_string_lossy());
                }
            }
            Err(error) => {
                let message = format!(
                    "Failed to create host configuration file {}: {}",
                    hosts_file_path.to_string_lossy(),
                    error
                );
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        let groups_config_file = fs::OpenOptions::new().write(true).create_new(true).open(groups_file_path.clone());
        match groups_config_file {
            Ok(mut file) => {
                if let Err(error) = file.write_all(DEFAULT_GROUPS_CONFIG.as_bytes()) {
                    let message = format!(
                        "Failed to write group configuration file {}: {}",
                        groups_file_path.to_string_lossy(),
                        error
                    );
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Created new group configuration file {}", groups_file_path.to_string_lossy());
                }
            }
            Err(error) => {
                let message = format!(
                    "Failed to create group configuration file {}: {}",
                    groups_file_path.to_string_lossy(),
                    error
                );
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        Ok(())
    }

    /// Writes the hosts.yml configuration file.
    pub fn write_hosts_config(config_dir: &String, hosts: &Hosts) -> io::Result<()> {
        let config_dir = if config_dir.is_empty() {
            file_handler::get_config_dir()
        }
        else {
            Path::new(config_dir).to_path_buf()
        };

        let hosts_file_path = config_dir.join(HOSTS_FILE);
        let hosts_config_file = fs::OpenOptions::new().write(true).truncate(true).open(hosts_file_path.clone());
        match hosts_config_file {
            Ok(mut file) => {
                let mut sanitized_hosts = hosts.clone();
                sanitized_hosts
                    .hosts
                    .values_mut()
                    .for_each(|host| host.effective = ConfigGroup::default());

                let hosts_config = serde_yaml::to_string(&sanitized_hosts)
                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

                if let Err(error) = file.write_all(hosts_config.as_bytes()) {
                    let message = format!("Failed to write host configuration file {}: {}", hosts_file_path.to_string_lossy(), error);
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Updated host configuration file {}", hosts_file_path.to_string_lossy());
                }
            }
            Err(error) => {
                let message = format!("Failed to open host configuration file {}: {}", hosts_file_path.to_string_lossy(), error);
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        Ok(())
    }

    /// Writes the groups.yml configuration file.
    pub fn write_groups_config(config_dir: &String, groups: &Groups) -> io::Result<()> {
        let config_dir = if config_dir.is_empty() {
            file_handler::get_config_dir()
        }
        else {
            Path::new(config_dir).to_path_buf()
        };

        let groups_file_path = config_dir.join(GROUPS_FILE);
        let groups_config_file = fs::OpenOptions::new().write(true).truncate(true).open(groups_file_path.clone());
        match groups_config_file {
            Ok(mut file) => {
                let groups_config = serde_yaml::to_string(groups)
                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

                if let Err(error) = file.write_all(groups_config.as_bytes()) {
                    let message = format!(
                        "Failed to write group configuration file {}: {}",
                        groups_file_path.to_string_lossy(),
                        error
                    );
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Updated group configuration file {}", groups_file_path.to_string_lossy());
                }
            }
            Err(error) => {
                let message = format!(
                    "Failed to open group configuration file {}: {}",
                    groups_file_path.to_string_lossy(),
                    error
                );
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        Ok(())
    }

    /// Writes the config.yml configuration file.
    pub fn write_main_config(config_dir: &String, config: &Configuration) -> io::Result<()> {
        let config_dir = if config_dir.is_empty() {
            file_handler::get_config_dir()
        }
        else {
            Path::new(config_dir).to_path_buf()
        };

        let main_config_file_path = config_dir.join(MAIN_CONFIG_FILE);
        let main_config_file = fs::OpenOptions::new().write(true).truncate(true).open(main_config_file_path.clone());
        match main_config_file {
            Ok(mut file) => {
                // Display options are currently not really user-configurable.
                let mut actual_display_options = get_default_main_config().display_options;
                // Exceptions. Allow some to be configurable.
                actual_display_options.show_status_bar = config.display_options.show_status_bar;

                let config_without_display_options = Configuration {
                    preferences: config.preferences.clone(),
                    cache_settings: config.cache_settings.clone(),
                    display_options: actual_display_options,
                    schema_version: config.schema_version.clone(),
                };

                let main_config = serde_yaml::to_string(&config_without_display_options)
                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;

                if let Err(error) = file.write_all(main_config.as_bytes()) {
                    let message = format!(
                        "Failed to write main configuration file {}: {}",
                        main_config_file_path.to_string_lossy(),
                        error
                    );
                    return Err(io::Error::new(io::ErrorKind::Other, message));
                }
                else {
                    log::info!("Updated main configuration file {}", main_config_file_path.to_string_lossy());
                }
            }
            Err(error) => {
                let message = format!(
                    "Failed to open main configuration file {}: {}",
                    main_config_file_path.to_string_lossy(),
                    error
                );
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }

        Ok(())
    }

    /// Helps keep the configuration up-to-date.
    pub fn upgrade_schema(main_config: &mut Configuration, groups_config: &mut Groups) {
        let default_groups = get_default_config_groups();
        let mut schema_version = main_config.schema_version.unwrap_or_else(|| 1);
        let old_version = schema_version;

        while schema_version < CURRENT_SCHEMA_VERSION {
            // NOTE: Default config groups should rarely be removed since they are used in older schema upgrades.
            match schema_version {
                1 => {
                    groups_config
                        .groups
                        .entry(String::from("nixos"))
                        .or_insert(default_groups.groups["nixos"].to_owned());
                },
                _ => {}
            }

            schema_version += 1;
        }

        if old_version < schema_version {
            main_config.schema_version = Some(schema_version);

            log::info!("Upgraded configuration schema from version {} to {}", old_version, schema_version);
        }
    }

    pub fn is_schema_outdated(schema_version: Option<u16>) -> bool {
        if let Some(version) = schema_version {
            version < CURRENT_SCHEMA_VERSION
        }
        else {
            true
        }
    }

    fn is_default<T: Default + PartialEq>(t: &T) -> bool {
        t == &T::default()
    }

    fn always<T>(_t: &T) -> bool {
        true
    }

    pub fn version_is_latest(version: &str) -> bool {
        version == "latest"
    }
}

pub fn get_default_config_groups() -> Groups {
    serde_yaml::from_str(DEFAULT_GROUPS_CONFIG)
        .expect("Default groups configuration is invalid YAML")
}

pub fn get_default_main_config() -> Configuration {
    serde_yaml::from_str(DEFAULT_MAIN_CONFIG)
        .expect("Default main configuration is invalid YAML")
}
