use std::collections::HashMap;
use serde_derive::{ Serialize, Deserialize };

use crate::enums::HostStatus;
use crate::host::Host;
use crate::host_manager::HostState;
use crate::module::command::CommandResult;
use crate::module::monitoring::MonitoringData;
use crate::utils::ErrorMessage;


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
 
#[derive(Clone, Serialize, Deserialize)]
pub struct HostDisplayData {
    pub host_state: HostState,
    pub new_monitoring_data: Option<(u64, MonitoringData)>,
    pub new_command_result: Option<(u64, CommandResult)>,
    pub new_errors: Vec<ErrorMessage>,
    /// Verification requests from connectors. Usually for key verification.
    /// Commands can already request (more diverse) user input so they don't use this.
    pub verification_requests: Vec<VerificationRequest>,
    pub stop: bool,
}

impl HostDisplayData {
    pub fn stop() -> Self {
        HostDisplayData {
            stop: true,
            ..Default::default()
        }
    }
}

impl Default for HostDisplayData {
    fn default() -> Self {
        HostDisplayData {
            host_state: HostState {
                host: Host::new("", &String::from("127.0.0.1"), &String::new(), &Vec::new()).unwrap(),
                status: HostStatus::default(),
                just_initialized: false,
                just_initialized_from_cache: false,
                is_initialized: false,
                monitor_data: HashMap::new(),
                command_results: HashMap::new(),
                monitor_invocations: HashMap::new(),
                command_invocations: HashMap::new(),
            },
            new_monitoring_data: None,
            new_command_result: None,
            new_errors: Vec::new(),
            verification_requests: Vec::new(),
            stop: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub source_id: String,
    pub message: String,
    /// Key fingerprint or similar identifier for the key.
    pub key_id: String,
}