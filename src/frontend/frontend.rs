use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use serde_derive::{ Serialize, Deserialize };

use crate::module::PlatformInfo;
use crate::module::command::CommandResult;
use crate::module::monitoring::MonitoringData;
use crate::enums::HostStatus;

pub trait Frontend {
    fn draw(display_data: &DisplayData);
}

#[derive(Default, Clone)]
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
 
// TODO: Use HostState instead?
#[derive(Clone, Serialize, Deserialize)]
pub struct HostDisplayData {
    pub name: String,
    pub domain_name: String,
    pub platform: PlatformInfo,
    pub status: HostStatus,
    pub ip_address: IpAddr,
    pub monitoring_data: HashMap<String, MonitoringData>,
    pub new_monitoring_data: Option<MonitoringData>,
    pub command_results: HashMap<String, CommandResult>,
    pub new_command_results: Option<CommandResult>,
    pub new_errors: Vec<String>,
    pub just_initialized: bool,
    pub just_initialized_from_cache: bool,
    pub is_initialized: bool,
    pub exit_thread: bool,
}

impl HostDisplayData {
    pub fn exit_token() -> Self {
        HostDisplayData {
            exit_thread: true,
            ..Default::default()
        }
    }
}

impl Default for HostDisplayData {
    fn default() -> Self {
        HostDisplayData {
            name: String::new(),
            domain_name: String::new(),
            platform: PlatformInfo::new(),
            status: HostStatus::Down,
            ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            monitoring_data: HashMap::new(),
            new_monitoring_data: None,
            command_results: HashMap::new(),
            new_command_results: None,
            new_errors: Vec::new(),
            just_initialized: false,
            just_initialized_from_cache: false,
            is_initialized: false,
            exit_thread: false,
        }
    }
}