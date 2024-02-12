
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};

use serde_derive::{Deserialize, Serialize};

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
    frontend_state_sender: Arc<Mutex<Vec<mpsc::Sender<frontend::HostDisplayData>>>>,
}

impl HostManager {
    pub fn new() -> HostManager {
        let hosts = Arc::new(Mutex::new(HostStateCollection::new()));
        let frontend_state_sender = Arc::new(Mutex::new(Vec::new()));

        HostManager {
            hosts: hosts,
            data_sender_prototype: None,
            data_receiver: None,
            receiver_thread: None,
            frontend_state_sender: frontend_state_sender,
        }
    }

    pub fn configure(&mut self, config: &configuration::Hosts) {
        if self.receiver_thread.is_some() {
            self.stop();
        }
        let mut hosts = self.hosts.lock().unwrap();
        hosts.clear();

        for (host_id, host_config) in config.hosts.iter() {
            log::debug!("Found configuration for host {}", host_id);

            let host = match Host::new(&host_id, &host_config.address, &host_config.fqdn, &host_config.settings.clone()) {
                Ok(host) => host,
                Err(error) => {
                    log::error!("{}", error);
                    continue;
                }
            };
            if let Err(error) = hosts.add(host.clone(), HostStatus::Pending) {
                log::error!("{}", error.to_string());
                continue;
            };
        }

        let (sender, receiver) = mpsc::channel::<StateUpdateMessage>();
        self.data_sender_prototype = Some(sender);
        self.data_receiver = Some(receiver);
    }

    pub fn stop(&mut self) {
        self.new_state_update_sender()
            .send(StateUpdateMessage::stop())
            .unwrap_or_else(|error| log::error!("Couldn't send stop command to state manager: {}", error));

        self.join();
    }

    /// Exit will forward the exit request to relevant parties that this component is responsible for 
    pub fn exit(&mut self) {
        self.new_state_update_sender()
            .send(StateUpdateMessage::stop())
            .unwrap_or_else(|error| log::error!("Couldn't send exit command to state manager: {}", error));

        self.join();
    }

    pub fn join(&mut self) {
        if let Some(thread) = self.receiver_thread.take() {
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

    pub fn add_observer(&mut self, sender: mpsc::Sender<frontend::HostDisplayData>) {
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
        observers: Arc<Mutex<Vec<mpsc::Sender<frontend::HostDisplayData>>>>) -> thread::JoinHandle<()> {

        thread::spawn(move || {
            loop {
                let state_update = match receiver.recv() {
                    Ok(data) => data,
                    Err(error) => {
                        log::error!("Stopped receiver thread: {}", error);
                        return;
                    }
                };

                if state_update.stop {
                    log::debug!("Restarting state manager thread.");
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
                let mut new_monitoring_data: Option<MonitoringData> = None;
                let mut new_command_results: Option<CommandResult> = None;

                if let Some(message_data_point) = state_update.data_point {

                    // Specially structured data point for passing platform info here.
                    if message_data_point.is_internal() {
                        if let Ok(platform) = Self::read_platform_info(&message_data_point) {
                            host_state.host.platform = platform;
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
                        // Check first if there already exists a key for monitor id.
                        if let Some(monitoring_data) = host_state.monitor_data.get_mut(&state_update.module_spec.id) {

                            monitoring_data.values.push_back(message_data_point.clone());

                            if monitoring_data.values.len() > DATA_POINT_BUFFER_SIZE {
                                monitoring_data.values.pop_front();
                            }
                        }
                        else {
                            let mut new_data = MonitoringData::new(state_update.module_spec.id.clone(), state_update.display_options);
                            new_data.values.push_back(message_data_point.clone());
                            host_state.monitor_data.insert(state_update.module_spec.id.clone(), new_data);
                        }

                        // Also add to a list of new data points.
                        let mut new = host_state.monitor_data.get(&state_update.module_spec.id).unwrap().clone();
                        new.values = VecDeque::from(vec![message_data_point.clone()]);
                        new_monitoring_data = Some(new);
                    }
                }
                else if let Some(command_result) = state_update.command_result {
                    host_state.command_results.insert(state_update.module_spec.id, command_result.clone());
                    // Also add to a list of new command results.
                    new_command_results = Some(command_result);
                }

                host_state.update_status();

                // Send the state update to the front end.
                for observer in observers.lock().unwrap().iter() {
                    observer.send(frontend::HostDisplayData {
                        host_state: host_state.clone(),
                        new_monitoring_data: new_monitoring_data.clone(),
                        new_command_results: new_command_results.clone(),
                        new_errors: state_update.errors.clone(),
                        ..Default::default()
                    }).unwrap();
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
                        true => format!("{}", monitor_data.display_options.display_text),
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

    fn read_platform_info(data_point: &DataPoint) -> Result<platform_info::PlatformInfo, String> {
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
                _ => return Err(String::from("Invalid platform info data"))
            }
        }
        Ok(platform)
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
    // Only used with MonitoringModule.
    pub data_point: Option<DataPoint>,
    pub command_result: Option<CommandResult>,
    pub errors: Vec<ErrorMessage>,
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
        }
    }

    fn update_status(&mut self) {
        let critical_monitor = &self.monitor_data.iter().find(|(_, data)| {
            // There should always be some monitoring data available at this point.
            data.is_critical && data.values.back().unwrap().criticality == Criticality::Critical
        });

        let pending_monitor = &self.monitor_data.iter().find(|(_, data)| {
            data.values.back().unwrap().criticality == Criticality::NoData
        });

        if let Some((name, _)) = critical_monitor {
            log::debug!("Host is now down since monitor \"{}\" is at critical level", name);
        }

        self.status = if critical_monitor.is_some() {
            HostStatus::Down
        }
        else if pending_monitor.is_some() {
            HostStatus::Pending
        }
        else {
            HostStatus::Up
        };

    }
}
