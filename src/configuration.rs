use serde_derive::{ Serialize, Deserialize };
use serde_yaml;
use std::fs;
use crate::utils::enums::HostStatus;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub general: General,
    pub authentication: Authentication,
    pub display_options: DisplayOptions,
    pub hosts: Vec<Host>,
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
pub struct Authentication {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub name: String,
    pub address: Option<String>,
    pub fqdn: Option<String>,
    pub monitors: Vec<MonitorConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorConfig {
    pub name: String,
    pub version: String,
    pub is_critical: Option<bool>,
}

impl Configuration {
    pub fn read(filename: &String) -> Result<Configuration, String> {
        let contents = fs::read_to_string(filename).map_err(|e| e.to_string())?;
        serde_yaml::from_str::<Configuration>(contents.as_str()).map_err(|e| e.to_string())
    }
}

