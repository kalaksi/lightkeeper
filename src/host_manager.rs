
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};

use serde_derive::{Deserialize, Serialize};

use crate::error::LkError;
use crate::frontend::frontend::VerificationRequest;
use crate::module::platform_info;
use crate::module::{
    ModuleSpecification,
    monitoring::MonitoringData,
    monitoring::DataPoint,
    command::CommandResult,
};

use crate::{
    enums::HostStatus,
    enums::Criticality,
    utils::VersionNumber,
    utils::ErrorMessage,
    host::Host,
    frontend,
    configuration,
};

const DATA_POINT_BUFFER_SIZE: usize = 4;


pub struct HostManager {
    hosts: Arc<Mutex<HostStateCollection>>,
    /// Provides sender handles for sending StateUpdateMessages to this instance.
    data_sender_prototype: Option<mpsc::Sender<StateUpdateMessage>>,
    data_receiver: Option<mpsc::Receiver<StateUpdateMessage>>,
    receiver_thread: Option<thread::JoinHandle<()>>,
    frontend_state_sender: Arc<Mutex<Vec<mpsc::Sender<frontend::UIUpdate>>>>,
}

impl HostManager {
    pub fn new() -> HostManager {
        let hosts = Arc::new(Mutex::new(HostStateCollection::new()));
        let frontend_state_sender = Arc::new(Mutex::new(Vec::new()));

        HostManager {
            hosts: hosts,
            frontend_state_sender: frontend_state_sender,
            data_receiver: None,
            data_sender_prototype: None,
            receiver_thread: None,
        }
    }

    pub fn configure(&mut self, config: &configuration::Hosts) {
        self.stop();

        let mut hosts = self.hosts.lock().unwrap();
        hosts.clear();

        // For certificate monitoring.
        let cert_monitor_host = Host::empty(crate::monitor_manager::CERT_MONITOR_HOST_ID, &vec![]);
        hosts.add(cert_monitor_host, HostStatus::Unknown).unwrap();

        // For regular host monitoring.
        for (host_id, host_config) in config.hosts.iter() {
            log::debug!("Found configuration for host {}", host_id);

            // TODO: UseSudo is currently always assumed.
            if let Ok(host) = Host::new(host_id, &host_config.address, &host_config.fqdn, &vec![crate::host::HostSetting::UseSudo]) {
                if let Err(error) = hosts.add(host, HostStatus::Unknown) {
                    log::error!("{}", error.to_string());
                    continue;
                }
            }
            else {
                log::error!("Failed to create host {}", host_id);
                continue;
            }
        }

        let (sender, receiver) = mpsc::channel::<StateUpdateMessage>();
        self.data_sender_prototype = Some(sender);
        self.data_receiver = Some(receiver);
    }

    pub fn stop(&mut self) {
        if let Some(thread) = self.receiver_thread.take() {
            self.new_state_update_sender()
                .send(StateUpdateMessage::stop())
                .unwrap_or_else(|error| log::error!("Couldn't send stop command to state manager: {}", error));

            thread.join().unwrap();
        }
    }

    /// Get Host details by name. Panics if the host is not found.
    pub fn get_host(&self, host_name: &String) -> Host {
        let hosts = self.hosts.lock().unwrap();
        hosts.hosts.get(host_name)
                   .unwrap_or_else(|| panic!("Host '{}' not found", host_name))
                   .host.clone()
    }

    pub fn new_state_update_sender(&self) -> mpsc::Sender<StateUpdateMessage> {
        self.data_sender_prototype.as_ref().unwrap().clone()
    }

    pub fn add_observer(&mut self, sender: mpsc::Sender<frontend::UIUpdate>) {
        self.frontend_state_sender.lock().unwrap().push(sender);
    }

    pub fn start_receiving_updates(&mut self) {
        let thread = Self::_start_receiving_updates(
            self.hosts.clone(),
            self.data_receiver.take().unwrap(),
            self.frontend_state_sender.clone(),
        );

        self.receiver_thread = Some(thread);
    }

    fn _start_receiving_updates(
        hosts: Arc<Mutex<HostStateCollection>>,
        receiver: mpsc::Receiver<StateUpdateMessage>,
        observers: Arc<Mutex<Vec<mpsc::Sender<frontend::UIUpdate>>>>) -> thread::JoinHandle<()> {

        thread::spawn(move || {
            log::debug!("Started receiving updates");

            loop {
                let state_update = match receiver.recv() {
                    Ok(data) => data,
                    Err(error) => {
                        log::error!("Stopped receiver thread: {}", error);
                        return;
                    }
                };

                if state_update.stop {
                    log::debug!("Gracefully stopping receiver thread");
                    return;
                }

                let mut hosts = hosts.lock().unwrap();
                let host_state = match hosts.hosts.get_mut(&state_update.host_name) {
                    Some(host_state) => host_state,
                    // It's possible that we receive state update from host that was just removed.
                    None => {
                        ::log::debug!("Host {} not found", state_update.host_name);
                        continue;
                    },
                };

                host_state.just_initialized = false;
                host_state.just_initialized_from_cache = false;
                let mut new_monitoring_data: Option<(u64, MonitoringData)> = None;
                let mut new_command_results: Option<(u64, CommandResult)> = None;

                if let Some(message_data_point) = state_update.data_point {
                    // Specially structured data point for passing platform info here.
                    if message_data_point.is_platform_info() {
                        host_state.monitor_invocations.remove(&state_update.invocation_id);

                        if let Ok((platform, ip_address)) = Self::read_platform_info(&message_data_point) {
                            host_state.host.platform = platform;
                            host_state.host.ip_address = ip_address;
                            log::debug!("[{}] Platform info updated", host_state.host.name);

                            // TODO: handle multiple platform info's.
                            if !message_data_point.is_from_cache {
                                host_state.just_initialized = true;
                                host_state.is_initialized = true;
                            }
                            else {
                                host_state.just_initialized_from_cache = true;
                            }
                        }
                        else {
                            log::error!("[{}] Invalid platform info received", host_state.host.name);
                        }
                    }
                    else {
                        // Initial NoData point will have invocation ID 0.
                        if state_update.invocation_id == 0 {
                            let mut new_data = MonitoringData::new(state_update.module_spec.id.clone(), state_update.display_options);
                            new_data.values.push_back(message_data_point.clone());
                            host_state.monitor_data.insert(state_update.module_spec.id.clone(), new_data);
                        }
                        else if message_data_point.criticality == Criticality::NoData {
                            host_state.monitor_invocations
                                .entry(state_update.invocation_id)
                                .or_insert(InvocationDetails::new(state_update.invocation_id, state_update.display_options.category));
                        }
                        else {
                            host_state.monitor_invocations.remove(&state_update.invocation_id);

                            // Monitoring data for platform info providers / internal modules won't exist in `monitor_data`.
                            if let Some(monitoring_data) = host_state.monitor_data.get_mut(&state_update.module_spec.id) {
                                monitoring_data.values.push_back(message_data_point.clone());

                                if monitoring_data.values.len() > DATA_POINT_BUFFER_SIZE {
                                    monitoring_data.values.pop_front();
                                }

                                // Also add to a list of new data points.
                                let mut new = host_state.monitor_data.get(&state_update.module_spec.id).unwrap().clone();
                                new.values = VecDeque::from(vec![message_data_point.clone()]);
                                new_monitoring_data = Some((state_update.invocation_id, new));
                            }
                        }
                    }
                }
                else if let Some(command_result) = state_update.command_result {
                    if command_result.criticality == Criticality::NoData {
                        host_state.command_invocations
                            .entry(state_update.invocation_id)
                            .or_insert(InvocationDetails::new(state_update.invocation_id, state_update.display_options.category));
                    }
                    else {
                        // Can be a partial result.
                        if command_result.progress < 100 {
                            host_state.command_invocations
                                .entry(state_update.invocation_id)
                                .and_modify(|invocation| invocation.progress = command_result.progress);
                        }
                        else {
                            host_state.command_invocations.remove(&state_update.invocation_id);
                        }
                        host_state.command_results.insert(state_update.module_spec.id, command_result.clone());
                        // Also add to a list of new command results.
                        new_command_results = Some((state_update.invocation_id, command_result));
                    }
                }
                else {
                    // Not all state updates will have valid data points or command results.
                    // Still need to remove invocation data.
                    match state_update.module_spec.module_type {
                        crate::module::ModuleType::Command => {
                            host_state.command_invocations.remove(&state_update.invocation_id);
                        },
                        crate::module::ModuleType::Monitor => {
                            host_state.monitor_invocations.remove(&state_update.invocation_id);
                        },
                        crate::module::ModuleType::Unknown |
                        crate::module::ModuleType::Connector => {},
                    }
                }

                host_state.update_status();

                let (verification_requests, unhandled_errors): (Vec<_>, Vec<_>) = state_update.errors.into_iter()
                    .partition(|error| error.kind == crate::error::ErrorKind::HostKeyNotVerified);

                let verification_requests: Vec<_> = verification_requests.iter()
                    .map(|error| VerificationRequest {
                        source_id: error.source_id.to_owned(),
                        key_id: error.parameter.to_owned().unwrap(),
                        message: error.message.to_owned(),
                    }) .collect();


                // Send the state update to the front end.
                for observer in observers.lock().unwrap().iter() {
                    observer.send(frontend::UIUpdate::Host(
                        frontend::HostDisplayData {
                            host_state: host_state.clone(),
                            new_monitoring_data: new_monitoring_data.clone(),
                            new_command_result: new_command_results.clone(),
                            new_errors: unhandled_errors.iter().cloned().map(ErrorMessage::from).collect(),
                            verification_requests: verification_requests.clone(),
                            ..Default::default()
                        })
                    ).unwrap();
                }
            }
        })
    }

    pub fn get_display_data(&self) -> frontend::DisplayData {
        let mut display_data = frontend::DisplayData::new();

        let hosts = self.hosts.lock().unwrap();
        for (_, host_state) in hosts.hosts.iter() {
            for (monitor_id, monitor_data) in host_state.monitor_data.iter() {
                if !display_data.all_monitor_names.contains(monitor_id) {
                    display_data.all_monitor_names.push(monitor_id.clone());

                    let header = match monitor_data.display_options.unit.is_empty() {
                        true => monitor_data.display_options.display_text.to_string(),
                        false => format!("{} ({})", monitor_data.display_options.display_text, monitor_data.display_options.unit),
                    };
                    display_data.table_headers.push(header);
                }
            }
        }

        for (host_name, state) in hosts.hosts.iter() {
            display_data.hosts.insert(host_name.clone(), frontend::HostDisplayData {
                host_state: state.clone(),
                ..Default::default()
            });
        }

        display_data.table_headers = vec![String::from("Status"), String::from("Name"), String::from("FQDN"), String::from("IP address")];
        display_data
    }

    fn read_platform_info(data_point: &DataPoint) -> Result<(platform_info::PlatformInfo, std::net::IpAddr), String> {
        let mut ip_address = std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0));
        let mut platform = platform_info::PlatformInfo::default();
        for data in data_point.multivalue.iter() {
            match data.label.as_str() {
                "os" => {
                    platform.os = platform_info::OperatingSystem::from_str(data.value.as_str()).map_err(|error| error.to_string())?
                },
                "os_version" => {
                    platform.os_version = VersionNumber::from_string(&data.value)
                },
                "os_flavor" => {
                    platform.os_flavor = platform_info::Flavor::from_str(data.value.as_str()).map_err(|error| error.to_string())?
                },
                "architecture" => {
                    platform.architecture = platform_info::Architecture::from(&data.value)
                },
                "ip_address" => {
                    ip_address = std::net::IpAddr::from_str(data.value.as_str()).map_err(|error| error.to_string())?
                },
                _ => return Err(String::from("Invalid platform info data"))
            }
        }

        Ok((platform, ip_address))
    }
}

impl Default for HostManager {
    fn default() -> Self {
        HostManager::new()
    }
}

#[derive(Default)]
pub struct StateUpdateMessage {
    pub host_name: String,
    pub display_options: frontend::DisplayOptions,
    pub module_spec: ModuleSpecification,
    /// Only used with monitors.
    pub data_point: Option<DataPoint>,
    /// Only used with commands.
    pub command_result: Option<CommandResult>,
    pub errors: Vec<LkError>,
    /// Unique invocation ID. Used as an identifier for asynchronously executed requests and received results.
    pub invocation_id: u64,
    /// Stops the receiver thread.
    pub stop: bool,
}

impl StateUpdateMessage {
    pub fn stop() -> Self {
        StateUpdateMessage {
            stop: true,
            ..Default::default()
        }
    }
}


struct HostStateCollection {
    hosts: HashMap<String, HostState>,
}

impl HostStateCollection {
    fn new() -> Self {
        HostStateCollection {
            hosts: HashMap::new(),
        }
    }

    fn add(&mut self, host: Host, default_status: HostStatus) -> Result<(), String> {
        if self.hosts.contains_key(&host.name) {
            return Err(String::from("Host already exists"));
        }

        self.hosts.insert(host.name.clone(), HostState::from_host(host, default_status));
        Ok(())
    }

    fn clear(&mut self) {
        self.hosts.clear();
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct HostState {
    pub host: Host,
    pub status: HostStatus,
    /// Host has received a real-time update for platform info (not a cached initial value).
    pub just_initialized: bool,
    pub just_initialized_from_cache: bool,
    pub is_initialized: bool,
    pub monitor_data: HashMap<String, MonitoringData>,
    pub command_results: HashMap<String, CommandResult>,
    /// Invocations in progress. Keeps track of monitor progress. Empty when all is done.
    pub monitor_invocations: HashMap<u64, InvocationDetails>,
    /// Invocations in progress. Keeps track of command progress. Empty when all is done.
    pub command_invocations: HashMap<u64, InvocationDetails>,
}

impl HostState {
    fn from_host(host: Host, status: HostStatus) -> Self {
        HostState {
            host: host,
            status: status,
            just_initialized: false,
            just_initialized_from_cache: false,
            is_initialized: false,
            monitor_data: HashMap::new(),
            command_results: HashMap::new(),
            monitor_invocations: HashMap::new(),
            command_invocations: HashMap::new(),
        }
    }

    fn update_status(&mut self) {
        // There should always be some monitoring data available at this point.
        let critical_monitor = self.monitor_data.iter()
            .find(|(_, data)| data.is_critical && data.values.back().unwrap().criticality == Criticality::Critical);

        // Status will be "pending" until all critical monitors have received data.
        let pending_critical_monitor = self.monitor_data.values()
            .find(|data| data.is_critical && data.values.back().unwrap().criticality == Criticality::NoData);

        let has_only_pending_monitors = self.monitor_data.values()
            .all(|data| data.values.iter().all(|datapoint| datapoint.criticality == Criticality::NoData));

        if let Some((name, _)) = critical_monitor {
            log::debug!("Host is now down since monitor \"{}\" is at critical level", name);
        }
        self.status = if critical_monitor.is_some() {
            HostStatus::Down
        }
        else if pending_critical_monitor.is_some() || has_only_pending_monitors {
            HostStatus::Pending
        }
        else {
            HostStatus::Up
        };

    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct InvocationDetails {
    pub invocation_id: u64,
    pub time: chrono::DateTime<chrono::Utc>,
    pub category: String,
    pub progress: u8,
}

impl InvocationDetails {
    pub fn new(invocation_id: u64, category: String) -> Self {
        InvocationDetails {
            invocation_id: invocation_id,
            time: chrono::Utc::now(),
            category: category,
            progress: 0,
        }
    }
}