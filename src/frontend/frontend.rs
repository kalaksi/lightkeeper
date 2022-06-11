use std::collections::HashMap;
use std::net::IpAddr;

use crate::module::monitoring::MonitoringData;
use crate::utils::enums::HostStatus;

pub trait Frontend {
    fn draw(display_data: &DisplayData);
}

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

pub struct HostDisplayData {
    pub name: String,
    pub domain_name: String,
    pub status: HostStatus,
    pub ip_address: IpAddr,
    pub monitoring_data: HashMap<String, MonitoringData>,
}


