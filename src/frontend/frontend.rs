use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use crate::module::command::CommandResult;
use crate::module::monitoring::MonitoringData;
use crate::utils::enums::HostStatus;

pub trait Frontend {
    fn draw(display_data: &DisplayData);
}

#[derive(Default)]
pub struct DisplayData {
    // Key is host name.
    pub hosts: HashMap<String, HostDisplayData>,
    pub all_monitor_names: Vec<String>,
    pub table_headers: Vec<String>,
}

impl DisplayData {
    pub fn new() -> Self {
        DisplayData {
            hosts: HashMap::new(),
            // To help creating tables.
            all_monitor_names: Vec::new(),
            table_headers: Vec::new(),
        }
    }
}
 
#[derive(Clone)]
pub struct HostDisplayData {
    pub name: String,
    pub domain_name: String,
    pub status: HostStatus,
    pub ip_address: IpAddr,
    pub monitoring_data: HashMap<String, MonitoringData>,
    pub command_results: HashMap<String, CommandResult>,
}

impl Default for HostDisplayData {
    fn default() -> Self {
        HostDisplayData {
            name: String::new(),
            domain_name: String::new(),
            status: HostStatus::Down,
            ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            monitoring_data: HashMap::new(),
            command_results: HashMap::new(),
        }
    }
}