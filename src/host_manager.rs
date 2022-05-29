use std::collections::HashMap;

use crate::module::{
    ModuleManager,
    monitoring::MonitoringData,
    connection::ConnectionModule,
    connection::AuthenticationDetails,
    ModuleSpecification,
};

use super::host::Host;

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

    pub fn get_connector(&mut self, host_name: &String, module_spec: &ModuleSpecification, authentication: Option<AuthenticationDetails>)
        -> Result<&mut Box<dyn ConnectionModule>, String>
    {
        let host_state = self.hosts.get(&host_name)?;
        log::info!("Connecting to {} ({}) with {}", host_name, host_state.host.socket_address, module_spec.id);

        if host_state.connections.contains_key(&module_spec.id) {
            return Ok(host_state.get_connection(&module_spec.id)?);
        }
        else {
            let mut connection = self.module_manager.new_connection_module(&module_spec);
            connection.connect(&host_state.host.socket_address, authentication)?;

            host_state.connections.insert(module_spec.id.clone(), connection);
            return Ok(host_state.get_connection(&module_spec.id)?);
        }
    }

    pub fn insert_monitoring_data(&mut self, host_name: &String, monitor_id: &String, data: MonitoringData) -> Result<(), String> {
        let host = self.hosts.get(host_name)?;

        if let Some(monitoring_data) = host.data.get_mut(monitor_id) {
            monitoring_data.push(data);
        }
        else {
            host.data.insert(monitor_id.clone(), vec![data]);
        }

        Ok(())
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

    fn get(&mut self, host_name: &String) -> Result<&mut HostState, String> {
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
