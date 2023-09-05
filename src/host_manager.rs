
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};

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
};

const DATA_POINT_BUFFER_SIZE: usize = 4;


pub struct HostManager {
    hosts: Arc<Mutex<HostCollection>>,
    /// Provides sender handles for sending StateUpdateMessages to this instance.
    data_sender_prototype: mpsc::Sender<StateUpdateMessage>,
    data_receiver: Option<mpsc::Receiver<StateUpdateMessage>>,
    receiver_thread: Option<thread::JoinHandle<()>>,
    frontend_state_sender: Arc<Mutex<Vec<mpsc::Sender<frontend::HostDisplayData>>>>,
}

impl HostManager {
    pub fn new() -> HostManager {
        let (sender, receiver) = mpsc::channel::<StateUpdateMessage>();
        let shared_hosts = Arc::new(Mutex::new(HostCollection::new()));
        let frontend_state_sender = Arc::new(Mutex::new(Vec::new()));

        HostManager {
            hosts: shared_hosts,
            data_sender_prototype: sender,
            data_receiver: Some(receiver),
            receiver_thread: None,
            frontend_state_sender: frontend_state_sender,
        }
    }

    pub fn configure(&mut self, config: &crate::configuration::Hosts) {
        for (host_id, host_config) in config.hosts.iter() {
            log::debug!("Found configuration for host {}", host_id);

            let host = match Host::new(&host_id, &host_config.address, &host_config.fqdn, &host_config.settings.clone()) {
                Ok(host) => host,
                Err(error) => {
                    log::error!("{}", error);
                    continue;
                }
            };
            let mut hosts = self.hosts.lock().unwrap();
            if let Err(error) = hosts.add(host.clone(), HostStatus::Pending) {
                log::error!("{}", error.to_string());
                continue;
            };
        }
    }

    pub fn join(&mut self) {
        self.receiver_thread.take().expect("Thread has already stopped.")
                            .join().unwrap();
    }

    pub fn add_host(&mut self, host: Host) -> Result<(), String> {
        let mut hosts = self.hosts.lock().unwrap();
        hosts.add(host, HostStatus::Pending)
    }

    /// Get Host details by name. Panics if the host is not found.
    pub fn get_host(&self, host_name: &String) -> Host {
        let hosts = self.hosts.lock().unwrap();
        hosts.hosts.get(host_name)
                   .expect(format!("Host '{}' not found", host_name).as_str())
                   .host.clone()
    }

    pub fn new_state_update_sender(&self) -> mpsc::Sender<StateUpdateMessage> {
        self.data_sender_prototype.clone()
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
        hosts: Arc<Mutex<HostCollection>>,
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

                if state_update.exit_thread {
                    log::debug!("Gracefully exiting state manager thread.");
                    for observer in observers.lock().unwrap().iter() {
                        observer.send(frontend::HostDisplayData::exit_token()).unwrap();
                    }
                    return;
                }

                let mut hosts = hosts.lock().unwrap();
                let host_state = hosts.hosts.get_mut(&state_update.host_name).unwrap();

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
                        name: host_state.host.name.clone(),
                        domain_name: host_state.host.fqdn.clone(),
                        platform: host_state.host.platform.clone(),
                        ip_address: host_state.host.ip_address.clone(),
                        monitoring_data: host_state.monitor_data.clone(),
                        new_monitoring_data: new_monitoring_data.clone(),
                        command_results: host_state.command_results.clone(),
                        new_command_results: new_command_results.clone(),
                        new_errors: state_update.errors.clone(),
                        status: host_state.status,
                        just_initialized: host_state.just_initialized,
                        just_initialized_from_cache: host_state.just_initialized_from_cache,
                        is_initialized: host_state.is_initialized,
                        exit_thread: false,
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
                name: state.host.name.clone(),
                domain_name: state.host.fqdn.clone(),
                platform: state.host.platform.clone(),
                ip_address: state.host.ip_address.clone(),
                monitoring_data: state.monitor_data.clone(),
                new_monitoring_data: None,
                command_results: state.command_results.clone(),
                new_command_results: None,
                new_errors: Vec::new(),
                status: state.status,
                just_initialized: state.just_initialized,
                just_initialized_from_cache: state.just_initialized_from_cache,
                is_initialized: state.is_initialized,
                exit_thread: false,
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
    pub exit_thread: bool,
}

impl StateUpdateMessage {
    pub fn exit_token() -> Self {
        StateUpdateMessage {
            exit_thread: true,
            ..Default::default()
        }
    }
}


struct HostCollection {
    hosts: HashMap<String, HostState>,
}

impl HostCollection {
    fn new() -> Self {
        HostCollection {
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
}


struct HostState {
    host: Host,
    status: HostStatus,
    /// Host has received a real-time update for platform info (not a cached initial value).
    just_initialized: bool,
    just_initialized_from_cache: bool,
    is_initialized: bool,
    monitor_data: HashMap<String, MonitoringData>,
    command_results: HashMap<String, CommandResult>,
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

        if let Some((name, _)) = critical_monitor {
            log::debug!("Host is now down since monitor \"{}\" is at critical level", name);
        }

        self.status = match critical_monitor {
            Some(_) => HostStatus::Down,
            None => HostStatus::Up,
        };
    }
}