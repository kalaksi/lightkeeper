/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::enums::HostStatus;
use crate::host::Host;
use crate::host_manager::HostState;
use crate::metrics::lmserver;
use crate::module::command::CommandResult;
use crate::module::monitoring::MonitoringData;
use crate::utils::ErrorMessage;


#[derive(Clone)]
pub enum UIUpdate {
    Host(HostDisplayData),
    Chart(lmserver::LMSResponse),
    /// Result for an invocation that runs on the desktop instead of the core (e.g. remote core file transfers).
    CommandResult(u64, CommandResult),
    CoreConnectionChanged(),
    FatalError(),
    Stop(),
}

impl Default for UIUpdate {
    fn default() -> Self {
        UIUpdate::Stop()
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
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
    /// Yes-no questions for the user, e.g. host key verification or overwriting a changed remote file.
    /// Commands can already request (more diverse) user input so they don't use this.
    pub confirmation_requests: Vec<ConfirmationRequest>,
}

impl Default for HostDisplayData {
    fn default() -> Self {
        HostDisplayData {
            host_state: HostState {
                host: Host::default(),
                status: HostStatus::default(),
                just_initialized: false,
                is_initialized: false,
                monitor_data: HashMap::new(),
                command_results: HashMap::new(),
                monitor_invocations: HashMap::new(),
                command_invocations: HashMap::new(),
            },
            new_monitoring_data: None,
            new_command_result: None,
            new_errors: Vec::new(),
            confirmation_requests: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    /// Error kind that caused the request, e.g. "HostKeyNotVerified" or "RemoteFileChanged".
    pub kind: String,
    pub invocation_id: u64,
    pub source_id: String,
    pub message: String,
    /// Kind-specific value, e.g. key fingerprint for host key verification.
    pub parameter: String,
}