use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};

use crate::Host;
use crate::module::connection::ResponseMessage;
use crate::module::monitoring::{ Monitor, DataPoint };
use crate::host_manager::StateUpdateMessage;
use crate::connection_manager::{ ConnectorRequest, ResponseHandlerCallback, RequestType };
use crate::utils::enums;


pub struct MonitorManager {
    // Monitor id is the second key.
    monitors: HashMap<Host, HashMap<String, Monitor>>,
    request_sender: Sender<ConnectorRequest>,
    // Channel to send state updates to HostManager.
    state_update_sender: Sender<StateUpdateMessage>,
}

impl MonitorManager {
    pub fn new(request_sender: mpsc::Sender<ConnectorRequest>, state_update_sender: Sender<StateUpdateMessage>) -> Self {
        MonitorManager {
            monitors: HashMap::new(),
            request_sender: request_sender,
            state_update_sender: state_update_sender,
        }
    }

    // Adds a monitor but only if a monitor with the same ID doesn't exist.
    pub fn add_monitor(&mut self, host: &Host, monitor: Monitor) {
        if !self.monitors.contains_key(host) {
            self.monitors.insert(host.clone(), HashMap::new());
        }

        let monitor_collection = self.monitors.get_mut(host).unwrap();
        let module_spec = monitor.get_module_spec();

        // Only add if missing.
        if !monitor_collection.contains_key(&module_spec.id) {
            log::debug!("Adding monitor {}", module_spec.id);

            // Add initial state value indicating no data as been received yet.
            Self::send_state_update(&host, &monitor, self.state_update_sender.clone(),
                                    DataPoint::value_with_level(String::from(""), enums::Criticality::NoData));

            monitor_collection.insert(module_spec.id, monitor);
        }
    }

    pub fn refresh_monitors(&self) {
        for (host, monitor_collection) in self.monitors.iter() {
            log::info!("Refreshing monitoring data for host {}", host.name);

            for (monitor_id, monitor) in monitor_collection.iter() {
                let messages = [monitor.get_connector_messages(), vec![monitor.get_connector_message()]].concat();

                // If monitor has a connector defined, send message through it, but
                // if there's no need for a connector, run the monitor independently.
                if let Some(connector_spec) = monitor.get_connector_spec() {
                    self.request_sender.send(ConnectorRequest {
                        connector_id: connector_spec.id,
                        source_id: monitor_id.clone(),
                        host: host.clone(),
                        messages: messages,
                        request_type: RequestType::Command,
                        response_handler: Self::get_response_handler(host.clone(), monitor.clone_module(), self.state_update_sender.clone())
                    }).unwrap_or_else(|error| {
                        log::error!("Couldn't send message to connector: {}", error);
                    });
                }
                else {
                    let handler = Self::get_response_handler(host.clone(), monitor.clone_module(), self.state_update_sender.clone());
                    handler(vec![Ok(ResponseMessage::empty())], false);
                } 
            }
        }
    }

    fn get_response_handler(host: Host, monitor: Monitor, state_update_sender: Sender<StateUpdateMessage>) -> ResponseHandlerCallback {
        Box::new(move |results, connector_is_connected| {
            let monitor_id = monitor.get_module_spec().id;
            let mut responses = Vec::<ResponseMessage>::new();
            let mut errors = Vec::<String>::new();

            for result in results {
                match result {
                    Ok(response) => responses.push(response),
                    Err(error) => errors.push(error),
                }
            }

            let mut result = DataPoint::empty_and_critical();
            if !responses.is_empty() {
                let process_result = match monitor.uses_multiple_commands() {
                    true => monitor.process_responses(host.clone(), responses, connector_is_connected),
                    false => monitor.process_response(host.clone(), responses.remove(0), connector_is_connected),
                };

                match process_result {
                    Ok(data_point) => {
                        log::debug!("Data point received for monitor {}: {} {}", monitor_id, data_point.label, data_point);
                        result = data_point;
                    },
                    Err(error) => {
                        log::error!("Error from monitor {}: {}", monitor_id, error);
                    }
                }
            }

            for error in errors {
                log::error!("Error refreshing monitor {}: {}", monitor_id, error);
            }

            Self::send_state_update(&host, &monitor, state_update_sender, result);
        })
    }

    fn send_state_update(host: &Host, monitor: &Monitor, state_update_sender: Sender<StateUpdateMessage>, data_point: DataPoint) {
        state_update_sender.send(StateUpdateMessage {
            host_name: host.name.clone(),
            display_options: monitor.get_display_options(),
            module_spec: monitor.get_module_spec(),
            data_point: Some(data_point),
            command_result: None,
        }).unwrap_or_else(|error| {
            log::error!("Couldn't send message to state manager: {}", error);
        });

    }

}