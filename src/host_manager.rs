use std::collections::HashMap;

use crate::module::monitoring::MonitoringModule;
use crate::utils::enums::HostStatus;
use crate::module::{
    ModuleManager,
    Module,
    monitoring::MonitoringData,
    monitoring::DataPoint,
    monitoring::Criticality,
    monitoring::DisplayOptions,
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

    pub fn get_connector(&mut self, host_name: &String, module_spec: &ModuleSpecification, settings: &HashMap<String, String>)
        -> Result<&mut Box<dyn connection::ConnectionModule>, String>
    {
        let host_state = self.hosts.get_mut(&host_name)?;

        if host_state.connections.contains_key(&module_spec.id) {
            return Ok(host_state.get_connection(&module_spec.id)?);
        }
        else {
            let mut connection = self.module_manager.new_connection_module(module_spec, settings);

            // If module does not have a connection dependency, it will be empty and a no-op.
            if connection.get_module_spec() != connection::Empty::get_metadata().module_spec {
                log::info!("Connecting to {} ({}) with {}", host_name, host_state.host.ip_address, module_spec.id);
                connection.connect(&host_state.host.ip_address)?;
            }

            host_state.connections.insert(module_spec.id.clone(), connection);
            return Ok(host_state.get_connection(&module_spec.id)?);
        }
    }

    pub fn insert_monitoring_data(&mut self, host_name: &String, monitor: &mut Box<dyn MonitoringModule>, data_point: DataPoint) -> Result<(), String> {
        let spec = &monitor.get_module_spec();

        log::debug!("New data point for {}: {}: {}", host_name, spec.id, data_point);
        let host_state = self.hosts.get_mut(&host_name)?;

        loop {
            if let Some(monitoring_data) = host_state.monitor_data.get_mut(&spec.id) {
                monitoring_data.values.push(data_point);
                break;
            }
            else {
                host_state.monitor_data.insert(spec.id.clone(), MonitoringData::new(monitor.get_display_options()));
            }
        }

        host_state.update_status();
        Ok(())
    }

    pub fn get_display_data(&self, excluded_monitors: &Vec<String>) -> frontend::DisplayData {
        let mut display_data = frontend::DisplayData::new();
        display_data.table_headers = vec![String::from("Status"), String::from("Name"), String::from("FQDN"), String::from("IP address")];

        for (_, host_state) in self.hosts.hosts.iter() {
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

        for (host_name, state) in self.hosts.hosts.iter() {
            let mut monitoring_data: HashMap<String, &MonitoringData> = HashMap::new();

            for (monitor_id, data) in state.monitor_data.iter() {
                if !excluded_monitors.contains(monitor_id) {
                    monitoring_data.insert(monitor_id.clone(), &data);
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
        self.hosts.insert(host_name, HostState::from_host(host, default_status));
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
    connections: HashMap<String, Box<dyn connection::ConnectionModule>>,
    monitor_data: HashMap<String, MonitoringData>,
}

impl HostState {
    fn from_host(host: Host, status: HostStatus) -> Self {
        HostState {
            host: host,
            connections: HashMap::new(),
            monitor_data: HashMap::new(),
            status: status,
        }
    }

    fn get_monitor(&mut self, monitor_id: &String) -> Result<&mut MonitoringData, String> {
        self.monitor_data.get_mut(monitor_id).ok_or(String::from("No such monitor"))
    }

    fn get_connection(&mut self, connection_id: &String) -> Result<&mut Box<dyn connection::ConnectionModule>, String> {
        self.connections.get_mut(connection_id).ok_or(String::from("No such connection"))
    }

    fn update_status(&mut self) {
        let critical_monitor = &self.monitor_data.iter().find(|(monitor_name, data)| {
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
