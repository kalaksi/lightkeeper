use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};

use crate::Host;
use crate::module::monitoring::{ Monitor, DataPoint };
use crate::host_manager::DataPointMessage;
use crate::connection_manager::{ ConnectorRequest, ResponseHandlerCallback };


pub struct MonitorManager {
    // Monitor id is the second key.
    monitors: HashMap<Host, HashMap<String, Monitor>>,
    request_sender: Sender<ConnectorRequest>,
    state_update_sender: Sender<DataPointMessage>,
}

impl MonitorManager {
    pub fn new(request_sender: mpsc::Sender<ConnectorRequest>, state_update_sender: Sender<DataPointMessage>) -> Self {
        let monitors = HashMap::new();

        MonitorManager {
            monitors: monitors,
            request_sender: request_sender,
            state_update_sender: state_update_sender,
        }
    }

    // Adds a monitor but only if a monitor with the same ID doesn't exist.
    pub fn add_monitor(&mut self, host: &Host, monitor: Monitor) {
        loop {

            if let Some(monitor_handlers) = self.monitors.get_mut(&host) {
                let module_spec = monitor.get_module_spec();

                if let None = monitor_handlers.get_mut(&module_spec.id) {
                    log::debug!("Adding monitor {}", module_spec.id);

                    monitor_handlers.insert(module_spec.id, monitor);
                }

                break;
            }
            else {
                self.monitors.insert(host.clone(), HashMap::new());
            }
        }
    }

    pub fn refresh_monitors(&self) {
        for (host, monitor_collection) in self.monitors.iter() {
            log::info!("Refreshing monitoring data for host {}", host.name);

            for (monitor_id, monitor) in monitor_collection.iter() {
                // If monitor has a connector defined, send message through it, but
                // if there's no need for a connector, run the monitor independently.
                if let Some(connector_spec) = monitor.get_connector_spec() {

                    self.request_sender.send(ConnectorRequest {
                        connector_id: connector_spec.id,
                        source_id: monitor_id.clone(),
                        host: host.clone(),
                        message: monitor.get_connector_message(),
                        response_handler: Self::get_response_handler(host.clone(), monitor.clone_module(), self.state_update_sender.clone())
                    }).unwrap_or_else(|error| {
                        log::error!("Couldn't send message to connector: {}", error);
                    });
                }
                else {
                    // TODO: Use empty connectors to unify implementation.
                    let data_point = monitor.process_response(host.clone(), String::new(), false).unwrap_or_else(|error| {
                        log::error!("Error while running monitor: {}", error);
                        DataPoint::empty_and_critical()
                    });

                    self.state_update_sender.send(DataPointMessage {
                        host_name: host.name.clone(),
                        display_options: monitor.get_display_options(),
                        module_spec: monitor.get_module_spec(),
                        data_point: data_point
                    }).unwrap_or_else(|error| {
                        log::error!("Couldn't send message to state manager: {}", error);
                    });
                } 
            }
        }
    }

    fn get_response_handler(host: Host, monitor: Monitor, state_update_sender: Sender<DataPointMessage>) -> ResponseHandlerCallback {
        Box::new(move |output, connector_is_connected| {
            let data_point = monitor.process_response(host.clone(), output, connector_is_connected);

            match data_point {
                Ok(data_point) => {
                    log::debug!("Data point received: {}", data_point);

                    state_update_sender.send(DataPointMessage {
                        host_name: host.name.clone(),
                        display_options: monitor.get_display_options(),
                        module_spec: monitor.get_module_spec(),
                        data_point: data_point
                    }).unwrap_or_else(|error| {
                        log::error!("Couldn't send message to state manager: {}", error);
                    });
                },
                Err(error) => {
                    log::error!("Error from monitor: {}", error);
                }
            }
        })
    }

}