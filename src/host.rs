
use std::{
    net::IpAddr,
    net::Ipv4Addr,
    net::ToSocketAddrs,
    str::FromStr,
    hash::Hash,
};

use crate::module::PlatformInfo;

#[derive(Clone)]
pub struct Host {
    pub name: String,
    pub fqdn: String,
    pub ip_address: IpAddr,
    pub platform: PlatformInfo,
}

impl Host {
    pub fn new(name: &String, ip_address: &String, fqdn: &String) -> Result<Self, String> {
        let mut new = Host {
            name: name.clone(),
            fqdn: fqdn.clone(),
            ip_address: match Ipv4Addr::from_str(ip_address) {
                Ok(address) => IpAddr::V4(address),
                Err(error) => return Err(format!("{}", error)),
            },
            platform: PlatformInfo::new()
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
                // TODO: get rid of unwraps.
                self.ip_address = format!("{}:0", self.fqdn).to_socket_addrs().unwrap().next().unwrap().ip();
                return Ok(());
            }
        }
        Ok(())
    }
}

// Also intended to be used as HashMap key. Only name acts as the identifier.
impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Host { }

impl Hash for Host {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}