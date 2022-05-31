use std::collections::HashMap;
use std::net::IpAddr;
use std::fmt;

use crate::{ module::monitoring::MonitoringData };

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

pub enum HostStatus {
    Up,
    Down,
}

impl fmt::Display for HostStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HostStatus::Up => write!(f, "Up"),
            HostStatus::Down => write!(f, "Up"),
        }
    }
}