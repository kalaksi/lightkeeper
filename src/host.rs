
use serde_derive::{ Serialize, Deserialize };
use std::{
    net::IpAddr,
    net::Ipv4Addr,
    net::ToSocketAddrs,
    str::FromStr,
};

use crate::module::PlatformInfo;

#[derive(Clone, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub fqdn: String,
    pub ip_address: IpAddr,
    pub platform: PlatformInfo,
    pub settings: Vec<HostSetting>,
}

impl Host {
    pub fn new(name: &str, ip_address: &str, fqdn: &str, settings: &[HostSetting]) -> Result<Self, String> {
        let mut new = Host {
            name: name.to_string(),
            fqdn: fqdn.to_string(),
            ip_address: match Ipv4Addr::from_str(ip_address) {
                Ok(address) => IpAddr::V4(address),
                Err(error) => return Err(error.to_string()),
            },
            platform: PlatformInfo::new(),
            settings: settings.to_vec(),
        };

        new.resolve_ip()?;
        Ok(new)
    }

    // Make sure IP address is defined by resolving FQDN if IP address is missing.
    pub fn resolve_ip(&mut self) -> Result<(), String> {
        if self.ip_address.is_unspecified() {
            if self.fqdn.is_empty() {
                return Err(format!("Host {} does not have FQDN or IP address defined.", self.name));
            }
            else {
                // Resolve FQDN and get the first IP address.
                let mut addresses = match format!("{}:0", self.fqdn).to_socket_addrs() {
                    Ok(addresses) => addresses,
                    Err(error) => return Err(format!("Failed to resolve: {}", error)),
                };

                if addresses.len() > 0 {
                    self.ip_address = addresses.next().unwrap().ip();
                }
                else {
                    return Err(String::from("Failed to resolve: No addresses found."));
                }
                return Ok(());
            }
        }
        Ok(())
    }
}

impl Default for Host {
    fn default() -> Self {
        Host {
            name: String::default(),
            fqdn: String::default(),
            ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            platform: PlatformInfo::default(),
            settings: Vec::default(),
        }
    }
}

/// Host settings should be controlled only through configuration files.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HostSetting {
    None,
    #[default]
    /// Use sudo for commands that require higher privileges.
    UseSudo,
}