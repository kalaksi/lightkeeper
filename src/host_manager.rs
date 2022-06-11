
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::sync::{Arc, Mutex};

use crate::module::ModuleSpecification;
use crate::module::monitoring::{
    MonitoringData,
    DataPoint,
    Criticality,
    DisplayOptions,
};

use crate::{
    utils::enums::HostStatus,
    host::Host,
    frontend,
};

pub struct HostManager {
    hosts: Arc<Mutex<HostCollection>>,
    data_sender_prototype: mpsc::Sender<DataPointMessage>,
    receiver_handle: Option<thread::JoinHandle<()>>,
}

// TODO rename to state-something
impl HostManager {
    pub fn new() -> HostManager {
        let (sender, receiver) = mpsc::channel::<DataPointMessage>();
        let shared_hosts = Arc::new(Mutex::new(HostCollection::new()));

        let handle = Self::process(shared_hosts.clone(), receiver);

        HostManager {
            hosts: shared_hosts,
            data_sender_prototype: sender,
            receiver_handle: Some(handle),
        }
    }

    pub fn add_host(&mut self, host: Host, default_status: HostStatus) -> Result<(), String> {
        let mut hosts = self.hosts.lock().unwrap();
        hosts.add(host, default_status)
    }

    pub fn join(&mut self) {
        self.receiver_handle.take().expect("Thread has already stopped.")
                            .join().unwrap();
    }

    pub fn get_state_udpate_channel(&self) -> Sender<DataPointMessage> {
        self.data_sender_prototype.clone()
    }

    fn process(hosts: Arc<Mutex<HostCollection>>, receiver: mpsc::Receiver<DataPointMessage>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                let message = receiver.recv().unwrap();

                let mut hosts = hosts.lock().unwrap();
                if let Some(host_state) = hosts.hosts.get_mut(&message.host_name) {
                    if let Some(monitoring_data) = host_state.monitor_data.get_mut(&message.module_spec.id) {
                        monitoring_data.values.push(message.data_point);
                    }
                    else {
                        let mut new_data = MonitoringData::new(message.display_options);
                        new_data.values.push(message.data_point);
                        host_state.monitor_data.insert(message.module_spec.id, new_data);
                    }

                    host_state.update_status();
                }
                else {
                    log::error!("Data for host {} does not exist.", message.host_name);
                }
            }
        })
    }

    pub fn get_display_data(&self, excluded_monitors: &Vec<String>) -> frontend::DisplayData {
        let mut display_data = frontend::DisplayData::new();
        display_data.table_headers = vec![String::from("Status"), String::from("Name"), String::from("FQDN"), String::from("IP address")];


        let hosts = self.hosts.lock().unwrap();
        for (_, host_state) in hosts.hosts.iter() {
            for (monitor_id, monitor_data) in host_state.monitor_data.iter() {
                if !display_data.all_monitor_names.contains(monitor_id) {
                    display_data.all_monitor_names.push(monitor_id.clone());

                    let header = match monitor_data.display_options.unit.is_empty() {
                        true => format!("{}", monitor_data.display_options.display_name),
                        false => format!("{} ({})", monitor_data.display_options.display_name, monitor_data.display_options.unit),
                    };
                    display_data.table_headers.push(header);
                }
            }
        }

        for (host_name, state) in hosts.hosts.iter() {
            let mut monitoring_data: HashMap<String, MonitoringData> = HashMap::new();

            for (monitor_id, data) in state.monitor_data.iter() {
                if !excluded_monitors.contains(monitor_id) {
                    monitoring_data.insert(monitor_id.clone(), data.clone());
                }
            }

            display_data.hosts.insert(host_name.clone(), frontend::HostDisplayData {
                name: state.host.name.clone(),
                domain_name: state.host.fqdn.clone(),
                ip_address: state.host.ip_address.clone(),
                monitoring_data: monitoring_data,
                status: state.status,
            });
        }


        display_data
    }

}

pub struct DataPointMessage {
    pub host_name: String,
    pub display_options: DisplayOptions,
    pub module_spec: ModuleSpecification,
    pub data_point: DataPoint,
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
    monitor_data: HashMap<String, MonitoringData>,
}

impl HostState {
    fn from_host(host: Host, status: HostStatus) -> Self {
        HostState {
            host: host,
            monitor_data: HashMap::new(),
            status: status,
        }
    }

    fn update_status(&mut self) {
        let critical_monitor = &self.monitor_data.iter().find(|(_, data)| {
            // There should always be some monitoring data available at this point.
            data.is_critical && data.values.last().unwrap().criticality == Criticality::Critical
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
