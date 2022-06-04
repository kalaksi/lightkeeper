use std::collections::HashMap;
use std::hash::Hash;

use crate::utils::enums::HostStatus;
use crate::module::Module;
use crate::module::monitoring::Criticality;
use crate::module::{
    ModuleManager,
    monitoring::MonitoringData,
    ModuleSpecification,
    connection,
};

use crate::{
    host::Host,
    frontend,
};

pub struct HostManager<'a> {
    hosts: HostCollection,
    module_manager: &'a ModuleManager,
}

impl<'a> HostManager<'a> {
    pub fn new(module_manager: &ModuleManager) -> HostManager {
        HostManager {
            hosts: HostCollection::new(),
            module_manager: &module_manager,
        }
    }

    pub fn add_host(&mut self, host: Host, critical_monitors: Vec<String>, default_status: HostStatus) -> Result<(), String> {
        self.hosts.add(host, critical_monitors, default_status)
    }

    pub fn get_host(&self, host_name: &String) -> Result<Host, String> {
        self.hosts.get(host_name).and_then(|host_state| Ok(host_state.host.clone()))
    }

    pub fn try_get_host(&self, host_name: &String) -> Option<Host> {
        self.hosts.hosts.get(host_name).and_then(|host_state| Some(host_state.host.clone()))
    }

    pub fn get_connector(&mut self, host_name: &String, module_spec: &ModuleSpecification, authentication: Option<connection::Credentials>)
        -> Result<&mut Box<dyn connection::ConnectionModule>, String>
    {
        let host_state = self.hosts.get_mut(&host_name)?;

        if host_state.connections.contains_key(&module_spec.id) {
            return Ok(host_state.get_connection(&module_spec.id)?);
        }
        else {
            let mut connection = self.module_manager.new_connection_module(&module_spec);

            // If module does not have a connection dependency, it will be empty and a no-op.
            if connection.get_module_spec() != connection::Empty::get_metadata().module_spec {
                log::info!("Connecting to {} ({}) with {}", host_name, host_state.host.ip_address, module_spec.id);
                connection.connect(&host_state.host.ip_address, authentication)?;
            }

            host_state.connections.insert(module_spec.id.clone(), connection);
            return Ok(host_state.get_connection(&module_spec.id)?);
        }
    }

    pub fn insert_monitoring_data(&mut self, host_name: &String, monitor_id: &String, data: MonitoringData) -> Result<(), String> {
        log::debug!("New monitoring data for {}: {}: {}", host_name, monitor_id, data);
        let host_state = self.hosts.get_mut(&host_name)?;

        if let Some(monitoring_data) = host_state.data.get_mut(monitor_id) {
            monitoring_data.push(data);
        }
        else {
            host_state.data.insert(monitor_id.clone(), vec![data]);
        }

        host_state.update_status();
        Ok(())
    }

    pub fn get_display_data(&self, excluded_monitors: &Vec<String>) -> frontend::DisplayData {
        let mut display_data = frontend::DisplayData::new();

        for (host_name, state) in self.hosts.hosts.iter() {

            let mut monitoring_data: HashMap<String, &Vec<MonitoringData>> = HashMap::new();
            for (monitor_name, data) in state.data.iter() {
                if !excluded_monitors.contains(monitor_name) {
                    monitoring_data.insert(monitor_name.clone(), &data);
                }
            }

            display_data.hosts.insert(host_name.clone(), frontend::HostDisplayData {
                name: &state.host.name,
                domain_name: &state.host.fqdn,
                ip_address: &state.host.ip_address,
                monitoring_data: monitoring_data,
                status: state.status,
            });
        }

        display_data
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

    fn add(&mut self, host: Host, critical_monitors: Vec<String>, default_status: HostStatus) -> Result<(), String> {
        if self.hosts.contains_key(&host.name) {
            return Err(String::from("Host already exists"));
        }

        let host_name = host.name.clone();
        self.hosts.insert(host_name, HostState::from_host(host, critical_monitors, default_status));
        Ok(())
    }

    fn get(&self, host_name: &String) -> Result<&HostState, String> {
        self.hosts.get(host_name).ok_or(String::from("No such host"))
    }

    fn get_mut(&mut self, host_name: &String) -> Result<&mut HostState, String> {
        self.hosts.get_mut(host_name).ok_or(String::from("No such host"))
    }

}


struct HostState {
    host: Host,
    status: HostStatus,
    critical_monitors: Vec<String>,
    connections: HashMap<String, Box<dyn connection::ConnectionModule>>,
    data: HashMap<String, Vec<MonitoringData>>,
}

impl HostState {
    fn from_host(host: Host, critical_monitors: Vec<String>, status: HostStatus) -> Self {
        HostState {
            host: host,
            connections: HashMap::new(),
            data: HashMap::new(),
            critical_monitors: critical_monitors,
            status: status,
        }
    }

    fn get_monitoring_data(&mut self, monitor_id: &String) -> Result<&mut Vec<MonitoringData>, String> {
        self.data.get_mut(monitor_id).ok_or(String::from("No such monitor"))
    }

    fn get_connection(&mut self, connection_id: &String) -> Result<&mut Box<dyn connection::ConnectionModule>, String> {
        self.connections.get_mut(connection_id).ok_or(String::from("No such connection"))
    }

    fn update_status(&mut self) {
        let critical_monitor = &self.data.iter().find(|(monitor_name, data)| {
            // There should always be some monitoring data available at this point.
            data.last().unwrap().criticality == Criticality::Critical && self.critical_monitors.contains(&monitor_name)
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
