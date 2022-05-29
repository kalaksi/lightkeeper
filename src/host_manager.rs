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
    hosts: HashMap<String, HostState<'a>>,
    module_manager: &'a ModuleManager,
}

impl<'a> HostManager<'a> {
    pub fn new(module_manager: &ModuleManager) -> HostManager {
        HostManager {
            hosts: HashMap::new(),
            module_manager: &module_manager,
        }
    }

    pub fn add_host(&mut self, host: Host<'a>)
    {
        let host_name = host.name.clone();
        let host_state = HostState {
            host: host,
            connections: HashMap::new(),
            data: Vec::new(),
        };

        self.hosts.insert(host_name, host_state);
    }

    pub fn get_connector(&mut self, host_name: &String, module_spec: &ModuleSpecification, authentication: Option<AuthenticationDetails>)
        -> Result<&mut Box<dyn ConnectionModule>, String>
    {
        if let Some(host_state) = self.hosts.get_mut(host_name) {
            log::info!("Connecting to {} ({}) with {}", host_name, host_state.host.socket_address, module_spec.id);

            if host_state.connections.contains_key(&module_spec.id) {
                return Ok(host_state.connections.get_mut(&module_spec.id).unwrap());
            }
            else {
                let mut connection = self.module_manager.new_connection_module(&module_spec);
                connection.connect(&host_state.host.socket_address, authentication)?;

                host_state.connections.insert(module_spec.id.clone(), connection);
                return Ok(host_state.connections.get_mut(&module_spec.id).unwrap());
            }
        }
        else {
            return Err(String::from("No such host"));
        }
    }

    pub fn insert_monitoring_data(&mut self, host_name: &String, data: MonitoringData) -> Result<(), String> {
        match self.hosts.get_mut(host_name) {
            Some(host) =>  {
                host.data.push(data);
                return Ok(());
            },
            None => {
                return Err(String::from("No such host"));
            }
        }
    }

}

struct HostState<'a> {
    host: Host<'a>,
    connections: HashMap<String, Box<dyn ConnectionModule>>,
    data: Vec<MonitoringData>,
}