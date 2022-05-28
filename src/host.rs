use std::net:: {
    SocketAddr,
    IpAddr,
    Ipv4Addr,
};

use std::str::FromStr;

pub struct Host<'a> {
    pub name: String,
    pub domain_name: &'a str,
    pub socket_address: SocketAddr,
}

impl<'a> Host<'a> {
    pub fn new(name: &String) -> Self
    {
        Host {
            name: name.clone(),
            domain_name: "",
            socket_address: SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                0,
            )
        }
    }

    pub fn set_address(&mut self, ip_address: &str, port: u16) {
        self.socket_address = match IpAddr::from_str(ip_address) {
            Ok(value) => SocketAddr::new(value, port),
            Err(_) => { log::error!("Invalid IP address \"{}\"", ip_address); return; }
        }
    }

    pub fn set_domain_name(&mut self, domain_name: &'a str) {
        self.domain_name = domain_name;
    }
}