use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};

use crate::Host;
use crate::module::connection::ResponseMessage;
use crate::module::monitoring::*;
use crate::host_manager::{StateUpdateMessage, HostManager};
use crate::connection_manager::{ ConnectorRequest, ResponseHandlerCallback, RequestType };


#[derive(Default)]
pub struct MonitorManager {
    // Host name is the first key, monitor id is the second key.
    monitors: HashMap<String, HashMap<String, Monitor>>,
    request_sender: Option<Sender<ConnectorRequest>>,
    // Channel to send state updates to HostManager.
    state_update_sender: Option<Sender<StateUpdateMessage>>,
    host_manager: Rc<RefCell<HostManager>>,
}

impl MonitorManager {
    pub fn new(request_sender: mpsc::Sender<ConnectorRequest>, host_manager: Rc<RefCell<HostManager>>) -> Self {
        MonitorManager {
            monitors: HashMap::new(),
            request_sender: Some(request_sender),
            host_manager: host_manager.clone(),
            state_update_sender: Some(host_manager.borrow().new_state_update_sender()),
        }
    }

    // Adds a monitor but only if a monitor with the same ID doesn't exist.
    pub fn add_monitor(&mut self, host: &Host, monitor: Monitor) {
        self.monitors.entry(host.name.clone()).or_insert(HashMap::new());

        let monitor_collection = self.monitors.get_mut(&host.name).unwrap();
        let module_spec = monitor.get_module_spec();

        // Only add if missing.
        if !monitor_collection.contains_key(&module_spec.id) {
            log::debug!("[{}] Adding monitor {}", host.name, module_spec.id);

            // Add initial state value indicating no data as been received yet.
            Self::send_state_update(&host, &monitor, self.state_update_sender.as_ref().unwrap().clone(),
                                    DataPoint::no_data());

            monitor_collection.insert(module_spec.id, monitor);
        }
    }

    pub fn refresh_platform_info(&self, host_id: Option<&String>) {
        for (host_name, monitor_collection) in self.monitors.iter() {
            if let Some(host_filter) = host_id {
                if host_name != host_filter {
                    continue;
                }
            }

            let host = self.host_manager.borrow().get_host(host_name);
            log::debug!("[{}] Refreshing platform info", host_name);

            // Executed only if required connector is available.
            if monitor_collection.iter().any(|(_, monitor)| monitor.get_connector_spec().unwrap_or_default().id == "ssh") {
                // TODO: remove hardcoding and execute once per connector type.
                let info_provider = internal::PlatformInfoSsh::new_monitoring_module(&HashMap::new());
                self.request_sender.as_ref().unwrap().send(ConnectorRequest {
                    connector_id: info_provider.get_connector_spec(),
                    source_id: info_provider.get_module_spec().id,
                    host: host.clone(),
                    messages: vec![info_provider.get_connector_message(host.clone())],
                    request_type: RequestType::Command,
                    response_handler: Self::get_response_handler(
                        host.clone(), info_provider, self.state_update_sender.as_ref().unwrap().clone()
                    )
                }).unwrap_or_else(|error| {
                    log::error!("Couldn't send message to connector: {}", error);
                });
            }
        }
    }

    /// Use `None` to refresh all monitors on every host or limit by host.
    pub fn refresh_monitors(&self, host_id: Option<&String>) {
        for (host_name, monitor_collection) in self.monitors.iter() {
            if let Some(host_filter) = host_id {
                if host_name != host_filter {
                    continue;
                }
            }

            let host = self.host_manager.borrow().get_host(host_name);

            log::info!("[{}] Refreshing monitoring data", host_name);
            if host.platform.is_unset() {
                log::warn!("[{}] Refreshing monitors despite missing platform info", host_name);
            }

            for (monitor_id, monitor) in monitor_collection.into_iter() {
                let messages = [monitor.get_connector_messages(host.clone()), vec![monitor.get_connector_message(host.clone())]].concat();
                self.request_sender.as_ref().unwrap().send(ConnectorRequest {
                    connector_id: monitor.get_connector_spec(),
                    source_id: monitor_id.clone(),
                    host: host.clone(),
                    messages: messages,
                    request_type: RequestType::Command,
                    response_handler: Self::get_response_handler(
                        host.clone(), monitor.box_clone(), self.state_update_sender.as_ref().unwrap().clone()
                    )
                }).unwrap_or_else(|error| {
                    log::error!("Couldn't send message to connector: {}", error);
                });
            }
        }
    }

    fn get_response_handler(host: Host, monitor: Monitor, state_update_sender: Sender<StateUpdateMessage>) -> ResponseHandlerCallback {
        Box::new(move |results| {
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
                    true => monitor.process_responses(host.clone(), responses),
                    false => monitor.process_response(host.clone(), responses.remove(0)),
                };

                match process_result {
                    Ok(data_point) => {
                        log::debug!("[{}] Data point received for monitor {}: {} {}", host.name, monitor_id, data_point.label, data_point);
                        result = data_point;
                    },
                    Err(error) => {
                        log::error!("[{}] Error from monitor {}: {}", host.name, monitor_id, error);
                    }
                }
            }

            for error in errors {
                log::error!("[{}] Error refreshing monitor {}: {}", host.name, monitor_id, error);
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