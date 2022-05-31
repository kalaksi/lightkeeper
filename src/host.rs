
use std::{
    net::IpAddr,
    net::Ipv4Addr,
};

#[derive(Clone)]
pub struct Host {
    pub name: String,
    pub fqdn: String,
    pub ip_address: IpAddr,
}

impl Host {
    pub fn new(name: &String) -> Self
    {
        Host {
            name: name.clone(),
            fqdn: String::new(),
            ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        }
    }
}