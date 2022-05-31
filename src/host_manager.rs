use std::collections::HashMap;

use crate::module::{
    ModuleManager,
    monitoring::MonitoringData,
    connection::ConnectionModule,
    connection::AuthenticationDetails,
    ModuleSpecification,
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

    pub fn add_host(&mut self, host: Host) -> Result<(), String> {
        self.hosts.add(host)
    }

    pub fn get_host(&self, host_name: &String) -> Result<Host, String> {
        self.hosts.get(host_name).and_then(|host_state| Ok(host_state.host.clone()))
    }

    pub fn try_get_host(&self, host_name: &String) -> Option<Host> {
        self.hosts.hosts.get(host_name).and_then(|host_state| Some(host_state.host.clone()))
    }

    pub fn get_connector(&mut self, host_name: &String, module_spec: &ModuleSpecification, authentication: Option<AuthenticationDetails>)
        -> Result<&mut Box<dyn ConnectionModule>, String>
    {
        let host_state = self.hosts.get_mut(&host_name)?;
        log::info!("Connecting to {} ({}) with {}", host_name, host_state.host.ip_address, module_spec.id);

        if host_state.connections.contains_key(&module_spec.id) {
            return Ok(host_state.get_connection(&module_spec.id)?);
        }
        else {
            let mut connection = self.module_manager.new_connection_module(&module_spec);
            connection.connect(&host_state.host.ip_address, authentication)?;

            host_state.connections.insert(module_spec.id.clone(), connection);
            return Ok(host_state.get_connection(&module_spec.id)?);
        }
    }

    pub fn insert_monitoring_data(&mut self, host_name: &String, monitor_id: &String, data: MonitoringData) -> Result<(), String> {
        log::debug!("{}: {}: {} {}", host_name, monitor_id, data.value, data.unit);
        let host = self.hosts.get_mut(host_name)?;

        if let Some(monitoring_data) = host.data.get_mut(monitor_id) {
            monitoring_data.push(data);
        }
        else {
            host.data.insert(monitor_id.clone(), vec![data]);
        }

        Ok(())
    }

    pub fn get_display_data(&self) -> frontend::DisplayData {
        let mut display_data = frontend::DisplayData::new();

        for (name, state) in self.hosts.hosts.iter() {
            display_data.hosts.insert(name.clone(), frontend::HostDisplayData {
                name: &state.host.name,
                status: frontend::HostStatus::Up,
                domain_name: &state.host.fqdn,
                ip_address: &state.host.ip_address,
                monitoring_data: &state.data,
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

    fn add(&mut self, host: Host) -> Result<(), String> {
        if self.hosts.contains_key(&host.name) {
            return Err(String::from("Host already exists"));
        }

        let host_name = host.name.clone();
        self.hosts.insert(host_name, HostState::from_host(host));
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
    connections: HashMap<String, Box<dyn ConnectionModule>>,
    data: HashMap<String, Vec<MonitoringData>>,
}

impl HostState {
    fn from_host(host: Host) -> Self {
        HostState {
            host: host,
            connections: HashMap::new(),
            data: HashMap::new(),
        }
    }

    fn get_monitoring_data(&mut self, monitor_id: &String) -> Result<&mut Vec<MonitoringData>, String> {
        self.data.get_mut(monitor_id).ok_or(String::from("No such monitor"))
    }

    fn get_connection(&mut self, connection_id: &String) -> Result<&mut Box<dyn ConnectionModule>, String> {
        self.connections.get_mut(connection_id).ok_or(String::from("No such connection"))
    }

}
