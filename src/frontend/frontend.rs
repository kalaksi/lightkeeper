use std::collections::HashMap;
use std::net::IpAddr;

use crate::{ module::monitoring::MonitoringData, utils::enums::HostStatus };

pub trait Frontend {
    fn draw(display_data: &DisplayData);
}

pub struct DisplayData<'a> {
    // Key is host name.
    pub hosts: HashMap<String, HostDisplayData<'a>>,
}

impl<'a> DisplayData<'a> {
    pub fn new() -> Self {
        DisplayData {
            hosts: HashMap::new(),
        }
    }
}

pub struct HostDisplayData<'a> {
    pub name: &'a String,
    pub domain_name: &'a String,
    pub status: HostStatus,
    pub ip_address: &'a IpAddr,
    pub monitoring_data: &'a HashMap<String, Vec<MonitoringData>>,
}
