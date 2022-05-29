use std::net:: {
    SocketAddr,
    IpAddr,
    Ipv4Addr,
};

use std::str::FromStr;

#[derive(Clone)]
pub struct Host {
    pub name: String,
    pub domain_name: String,
    pub socket_address: SocketAddr,
}

impl Host {
    pub fn new(name: &String) -> Self
    {
        Host {
            name: name.clone(),
            domain_name: String::new(),
            socket_address: SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                0,
            )
        }
    }

    pub fn set_address(&mut self, socket_address: &str) {
        self.socket_address = SocketAddr::from_str(socket_address)
                              .expect(format!("Invalid socket address '{}'", socket_address).as_str());
    }

    pub fn set_address_and_port(&mut self, ip_address: &str, port: u16) {
        self.socket_address = match IpAddr::from_str(ip_address) {
            Ok(value) => SocketAddr::new(value, port),
            Err(_) => { log::error!("Invalid IP address \"{}\"", ip_address); return; }
        }
    }

    pub fn set_domain_name(&mut self, domain_name: &String) {
        self.domain_name = domain_name.clone();
    }
}