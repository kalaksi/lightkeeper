
use tabled::Tabled;

use std::{
    net::IpAddr,
    net::Ipv4Addr,
    str::FromStr,
};

#[derive(Clone, Tabled)]
pub struct Host {
    pub name: String,
    pub domain_name: String,
    pub ip_address: IpAddr,
}

impl Host {
    pub fn new(name: &String) -> Self
    {
        Host {
            name: name.clone(),
            domain_name: String::new(),
            ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        }
    }

    pub fn set_address(&mut self, ip_address: &str) -> Result<&IpAddr, String> {
        self.ip_address = IpAddr::V4(Ipv4Addr::from_str(ip_address).map_err(|e| e.to_string())?);
        Ok(&self.ip_address)
    }

    pub fn set_domain_name(&mut self, domain_name: &String) {
        self.domain_name = domain_name.clone();
    }
}