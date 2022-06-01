use serde_derive::{ Serialize, Deserialize };
use toml;
use std::fs;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub authentication: Authentication,
    pub hosts: Vec<Host>,
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
    pub monitors: Vec<String>,
    pub critical: Option<bool>,
}

impl Configuration {
    pub fn read(filename: &String) -> Result<Configuration, String> {
        let contents = fs::read_to_string(filename).map_err(|e| e.to_string())?;
        toml::from_str::<Configuration>(contents.as_str()).map_err(|e| e.to_string())
    }
}