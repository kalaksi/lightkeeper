/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use crate::error::*;
use crate::module::connection::RequestResponse;
use crate::Host;
use crate::configuration::Hosts;
use crate::module::connection::ResponseMessage;
use crate::module::{monitoring::*, ModuleSpecification};
use crate::module::ModuleFactory;
use crate::host_manager::{StateUpdateMessage, HostManager};
use crate::connection_manager::{ ConnectorRequest, RequestType };

pub const CERT_MONITOR_HOST_ID: &str = "_cert-monitor";


// Default needs to be implemented because of Qt QObject requirements.
#[derive(Default)]
pub struct MonitorManager {
    // Host name is the first key, monitor id is the second key.
    monitors: Arc<Mutex<HashMap<String, HashMap<String, Monitor>>>>,
    platform_info_providers: Arc<Mutex<HashMap<String, Monitor>>>,
    /// For communication to ConnectionManager.
    request_sender: Option<mpsc::Sender<ConnectorRequest>>,
    // Channel to send state updates to HostManager.
    state_update_sender: Option<mpsc::Sender<StateUpdateMessage>>,
    /// Every refresh operation gets an invocation ID. Valid ID numbers begin from 1.
    invocation_id_counter: u64,

    // Shared resources. Only used for fetching up-to-date data.
    host_manager: Rc<RefCell<HostManager>>,
    module_factory: Arc<ModuleFactory>,

    response_sender_prototype: Option<mpsc::Sender<RequestResponse>>,
    response_receiver: Option<mpsc::Receiver<RequestResponse>>,
    response_receiver_thread: Option<thread::JoinHandle<()>>,
}

impl MonitorManager {
    pub fn new(host_manager: Rc<RefCell<HostManager>>, module_factory: Arc<ModuleFactory>) -> Self {

        MonitorManager {
            host_manager: host_manager.clone(),
            module_factory: module_factory,
            ..Default::default()
        }
    }

    pub fn configure(&mut self,
                     hosts_config: &Hosts,
                     request_sender: mpsc::Sender<ConnectorRequest>,
                     state_update_sender: mpsc::Sender<StateUpdateMessage>) {

        // MonitorManager has almost no state so can be reinitialized fully.
        self.stop();

        // Just clearing is not enough if the mutex is poisoned.
        self.monitors = Arc::new(Mutex::new(HashMap::new()));
        self.platform_info_providers = Arc::new(Mutex::new(HashMap::new()));

        self.request_sender = Some(request_sender);
        self.state_update_sender = Some(state_update_sender);

        // Certificate monitors.
        if hosts_config.certificate_monitors.len() > 0 {
            log::debug!("Found configuration for {} certificate monitors", hosts_config.certificate_monitors.len());
        }

        let mut settings = HashMap::new();
        settings.insert("addresses".to_string(), hosts_config.certificate_monitors.join(","));
        let cert_monitor = internal::CertMonitor::new_monitoring_module(&settings);
        self.add_monitor(CERT_MONITOR_HOST_ID.to_string(), cert_monitor, true);

        // Regular host monitoring.
        for (host_id, host_config) in hosts_config.hosts.iter() {
            // Names prefixed with _ are reserved for internal use.
            if host_id.starts_with("_") {
                continue;
            }

            let mut new_monitors = Vec::<Monitor>::new();
            for (monitor_id, monitor_config) in host_config.effective.monitors.iter() {
                let monitor_spec = ModuleSpecification::monitor(monitor_id, &monitor_config.version);
                let monitor = match self.module_factory.new_monitor(&monitor_spec, &monitor_config.settings) {
                    Some(monitor) => monitor,
                    None => continue,
                };
                new_monitors.push(monitor);
            }

            let base_modules = new_monitors.iter().filter_map(|monitor| monitor.get_metadata_self().parent_module)
                                                  .collect::<Vec<_>>();

            for monitor in new_monitors {
                // Base modules won't get the initial NoData data point sent.
                let is_base = base_modules.contains(&monitor.get_module_spec());
                self.add_monitor(host_id.clone(), monitor, !is_base);
            }
        }

        let (sender, receiver) = mpsc::channel::<RequestResponse>();
        self.response_sender_prototype = Some(sender);
        self.response_receiver = Some(receiver);
    }

    pub fn stop(&mut self) {
        if let Some(thread) = self.response_receiver_thread.take() {
            if let Err(_) = self.new_response_sender().send(RequestResponse::stop()) {
                log::warn!("Couldn't stop thread, it may have already stopped.");
            }

            if let Err(_) = thread.join() {
                log::warn!("Thread had paniced");
            }
        }
    }

    pub fn new_response_sender(&self) -> mpsc::Sender<RequestResponse> {
        self.response_sender_prototype.clone().unwrap()
    }

    // Adds a monitor but only if a monitor with the same ID doesn't exist.
    fn add_monitor(&mut self, host_id: String, monitor: Monitor, send_initial_value: bool) {
        let Ok(mut monitors) = self.monitors.lock() else {
            self.send_state_update(StateUpdateMessage::fatal_error());
            return;
        };

        let monitor_collection = monitors.entry(host_id.clone()).or_insert(HashMap::new());
        let module_spec = monitor.get_module_spec();

        // Only add if missing.
        if !monitor_collection.contains_key(&module_spec.id) {

            if send_initial_value {
                // Add initial state value indicating no data as been received yet.
                self.send_state_update(StateUpdateMessage {
                    host_name: host_id,
                    display_options: monitor.get_display_options(),
                    module_spec: monitor.get_module_spec(),
                    data_point: Some(DataPoint::pending()),
                    invocation_id: 0,
                    ..Default::default()
                });
            }

            // For platform info modules.
            if monitor.get_connector_spec().unwrap_or_default().id == "ssh" {
                let mut platform_info_providers = self.platform_info_providers.lock().unwrap();
                let ssh_provider = internal::PlatformInfoSsh::new_monitoring_module(&HashMap::new());
                platform_info_providers.entry(ssh_provider.get_module_spec().id.clone()).or_insert(ssh_provider);
            }

            monitor_collection.insert(module_spec.id, monitor);
        }
    }

    /// Intended to be run only once in the beginning when possibly refreshing all host data.
    /// Returns list of host IDs that were refreshed.
    pub fn refresh_platform_info_all(&mut self) -> Vec<String> {
        let Ok(monitors) = self.monitors.lock() else {
            self.send_state_update(StateUpdateMessage::fatal_error());
            return Vec::new();
        };

        let host_ids = monitors.keys().cloned().collect::<Vec<_>>();
        drop(monitors);

        for host_id in host_ids.iter() {
            self.refresh_platform_info(host_id);
        }

        host_ids
    }

    /// Refreshes platform info and such in preparation for actual monitor refresh.
    pub fn refresh_platform_info(&mut self, host_id: &String) {
        let Ok(monitors) = self.monitors.lock() else {
            self.send_state_update(StateUpdateMessage::fatal_error());
            return;
        };

        let platform_info_providers = self.platform_info_providers.lock().unwrap();

        // Internal modules start with an underscore.
        let monitors_for_host = monitors.iter()
            .filter(|(host_id_key, _)| &host_id == host_id_key && !host_id_key.starts_with("_"));

        for (host_name, monitor_collection) in monitors_for_host {
            let mut host = self.host_manager.borrow().get_host(host_name);

            if let Err(error) = host.resolve_ip() {
                // TODO: show in UI?
                log::error!("Failed to resolve IP address for host {}: {}", host_name, error);
            }

            for info_provider in platform_info_providers.values() {
                // Executed only if required connector is used on the host.
                if monitor_collection.values().all(|monitor|
                    monitor.get_connector_spec().unwrap_or_default().id != info_provider.get_connector_spec().unwrap().id
                ) {
                    continue;
                }

                let commands = match get_monitor_connector_messages(&host, &info_provider, &DataPoint::empty()) {
                    Ok(messages) => messages,
                    Err(error) => {
                        log::error!("Monitor failed: {}", error);
                        return;
                    }
                };

                self.invocation_id_counter += 1;

                // Notify host state manager about new pending monitor invocation.
                self.send_state_update(StateUpdateMessage {
                    host_name: host.name.clone(),
                    display_options: info_provider.get_display_options(),
                    module_spec: info_provider.get_module_spec(),
                    data_point: Some(DataPoint::pending()),
                    invocation_id: self.invocation_id_counter,
                    ..Default::default()
                });

                self.send_connector_request(ConnectorRequest {
                    connector_spec: info_provider.get_connector_spec(),
                    source_id: info_provider.get_module_spec().id,
                    host: host.clone(),
                    invocation_id: self.invocation_id_counter,
                    response_sender: self.new_response_sender(),
                    request_type: RequestType::MonitorCommand {
                        parent_datapoint: None,
                        extension_monitors: Vec::new(),
                        commands: commands,
                    },
                });
            }
        }
    }

    pub fn get_all_host_categories(&self, host_id: &String) -> Vec<String> {
        let Ok(monitors) = self.monitors.lock() else {
            self.send_state_update(StateUpdateMessage::fatal_error());
            return Vec::new();
        };

        let Some(host_monitors) = monitors.get(host_id) else {
            log::error!("Invalid host: {}", host_id);
            return Vec::new();
        };

        let mut categories = host_monitors.iter()
            .map(|(_, monitor)| monitor.get_display_options().category.clone())
            .collect::<Vec<_>>();

        categories.sort();
        categories.dedup();
        categories
    }

    pub fn refresh_certificate_monitors(&mut self) -> Vec<u64> {
        let Ok(monitors) = self.monitors.lock() else {
            self.send_state_update(StateUpdateMessage::fatal_error());
            return Vec::new();
        };

        let certificate_monitors = monitors[CERT_MONITOR_HOST_ID].iter().collect();
        let cert_monitor_host = self.host_manager.borrow().get_host(&CERT_MONITOR_HOST_ID.to_string());
        self.refresh_monitors(cert_monitor_host, certificate_monitors)
    }

    /// Returns the invocation IDs of the refresh operations.
    pub fn refresh_monitors_of_category(&mut self, host_id: &String, category: &String) -> Vec<u64> {
        let host = self.host_manager.borrow().get_host(host_id);
        let Ok(monitors) = self.monitors.lock() else {
            self.send_state_update(StateUpdateMessage::fatal_error());
            return Vec::new();
        };

        let Some(host_monitors) = monitors.get(host_id) else {
            log::error!("Invalid host: {}", host_id);
            return Vec::new();
        };

        let monitors_by_category = host_monitors.iter()
            .filter(|(_, monitor)| &monitor.get_display_options().category == category)
            .collect();

        let invocation_ids = self.refresh_monitors(host, monitors_by_category);
        self.invocation_id_counter += invocation_ids.len() as u64;
        invocation_ids
    }

    /// Refresh by monitor ID.
    /// Returns the invocation IDs of the refresh operations.
    pub fn refresh_monitors_by_id(&mut self, host_id: &String, monitor_id: &String) -> Vec<u64> {
        let host = self.host_manager.borrow().get_host(host_id);

        let Ok(monitors) = self.monitors.lock() else {
            self.send_state_update(StateUpdateMessage::fatal_error());
            return Vec::new();
        };

        let Some(host_monitors) = monitors.get(host_id) else {
            log::error!("Invalid host: {}", host_id);
            return Vec::new();
        };

        let monitor = host_monitors.iter()
            .filter(|(_, monitor)| &monitor.get_module_spec().id == monitor_id)
            .collect();

        let invocation_ids = self.refresh_monitors(host, monitor);
        self.invocation_id_counter += invocation_ids.len() as u64;
        invocation_ids
    }

    fn refresh_monitors(&self, host: Host, monitors: HashMap<&String, &Monitor>) -> Vec<u64> {
        if !host.platform.is_set() && monitors.values().any(|monitor| !monitor.is_internal()) {
            log::warn!("[{}] Refreshing monitors despite missing platform info", host.name);
        }

        let mut current_invocation_id = self.invocation_id_counter;
        let mut invocation_ids = Vec::new();

        // Split into 2: base modules and extension modules.
        let (extensions, bases): (Vec<&Monitor>, Vec<&Monitor>) = 
            monitors.values().partition(|monitor| monitor.get_metadata_self().parent_module.is_some());

        for monitor in bases {
            current_invocation_id += 1;
            invocation_ids.push(current_invocation_id);

            let extension_ids = extensions.iter()
                .filter(|ext| ext.get_metadata_self().parent_module.unwrap() == monitor.get_module_spec())
                .map(|ext| ext.get_module_spec().id.clone()).collect();

            // Notify host state manager about new pending monitor invocation.
            self.send_state_update(StateUpdateMessage {
                host_name: host.name.clone(),
                display_options: monitor.get_display_options(),
                module_spec: monitor.get_module_spec(),
                data_point: Some(DataPoint::pending()),
                invocation_id: current_invocation_id,
                ..Default::default()
            });

            let messages = match get_monitor_connector_messages(&host, &monitor, &DataPoint::empty()) {
                Ok(messages) => messages,
                Err(error) => {
                    log::error!("Monitor failed: {}", error);

                    self.send_state_update(StateUpdateMessage {
                        host_name: host.name.clone(),
                        display_options: monitor.get_display_options(),
                        module_spec: monitor.get_module_spec(),
                        errors: vec![error],
                        invocation_id: current_invocation_id,
                        ..Default::default()
                    });

                    continue;
                }
            };

            self.send_connector_request(ConnectorRequest {
                connector_spec: monitor.get_connector_spec(),
                source_id: monitor.get_module_spec().id,
                host: host.clone(),
                invocation_id: current_invocation_id,
                response_sender: self.new_response_sender(),
                request_type: RequestType::MonitorCommand {
                    parent_datapoint: None,
                    extension_monitors: extension_ids,
                    commands: messages,
                },
            });
        }

        invocation_ids
    }

    fn send_connector_request(&self, request: ConnectorRequest) {
        if let Err(error) = self.request_sender.as_ref().unwrap().send(request) {
            log::error!("Failed to send connector request: {}", error);
            self.send_state_update(StateUpdateMessage::fatal_error());
        }
    }

    fn send_state_update(&self, message: StateUpdateMessage) {
        // If upstream state manager has crashed for some reason, there's nothing we can do to recover.
        if let Err(error) = self.state_update_sender.as_ref().unwrap().send(message) {
            log::error!("Failed to send state update: {}", error);
            panic!("Failed to send state update: {}", error);
        }
    }

    //
    // RESPONSE HANDLING
    //

    pub fn start_processing_responses(&mut self) {
        let thread = Self::_start_processing_responses(
            self.monitors.clone(),
            self.platform_info_providers.clone(),
            self.request_sender.as_ref().unwrap().clone(),
            self.state_update_sender.as_ref().unwrap().clone(),
            self.response_sender_prototype.as_ref().unwrap().clone(),
            self.response_receiver.take().unwrap(),
        );

        self.response_receiver_thread = Some(thread);
    }

    fn _start_processing_responses(
        monitors: Arc<Mutex<HashMap<String, HashMap<String, Monitor>>>>,
        platform_info_providers: Arc<Mutex<HashMap<String, Monitor>>>,
        request_sender: mpsc::Sender<ConnectorRequest>,
        state_update_sender: mpsc::Sender<StateUpdateMessage>,
        response_sender: mpsc::Sender<RequestResponse>,
        response_receiver: mpsc::Receiver<RequestResponse>,
    ) -> thread::JoinHandle<()> {

        thread::spawn(move || {
            log::debug!("Started processing responses");

            loop {
                let response = match response_receiver.recv() {
                    Ok(response) => response,
                    Err(error) => {
                        log::error!("Stopped response receiver thread: {}", error);
                        break;
                    }
                };

                if response.stop {
                    break;
                }

                let results_len = response.responses.len();
                let (responses, errors): (Vec<_>, Vec<_>) =  response.responses.into_iter().partition(Result::is_ok);
                let responses = responses.into_iter().map(Result::unwrap).collect::<Vec<_>>();
                let errors = errors.into_iter().map(Result::unwrap_err).collect::<Vec<_>>();

                let Ok(monitors) = monitors.lock() else {
                    if let Err(error) = state_update_sender.send(StateUpdateMessage::fatal_error()) {
                        log::error!("Failed to send state update: {}", error);
                        panic!("Failed to send state update: {}", error);
                    }
                    continue;
                };

                let platform_info_providers = platform_info_providers.lock().unwrap();
                let monitor_id = &response.source_id;
                // Search from internal monitors first.
                let monitor = if let Some(monitor) = platform_info_providers.get(monitor_id) {
                    monitor
                }
                else if let Some(monitor) = monitors.get(&response.host.name).and_then(|m| m.get(monitor_id)) {
                    monitor
                }
                else {
                    // Can happen if the host or monitor was removed while command was in progress.
                    log::debug!("[{}][{}] Ignoring response for unknown host or monitor", response.host.name, monitor_id);
                    continue;
                };
                
                let (parent_datapoint, mut extension_monitors) = match response.request_type {
                    RequestType::MonitorCommand { parent_datapoint, extension_monitors, .. } => {
                        (parent_datapoint, extension_monitors)
                    },
                    _ => {
                        log::warn!("[{}][{}] Ignoring invalid datapoint", response.host.name, monitor_id);
                        continue; 
                    }
                };


                let mut datapoint_result;
                if results_len == 0 {
                    // Some special modules require no connectors and receive no response messages, such
                    // as `os`, which only uses existing platform info.
                    datapoint_result = monitor.process_response(response.host.clone(), ResponseMessage::empty(), DataPoint::empty());
                }
                else if responses.len() > 0 {
                    datapoint_result = monitor.process_responses(response.host.clone(), responses.clone(), parent_datapoint.clone().unwrap_or_default());

                    if let Err(error) = datapoint_result {
                        if error.is_empty() {
                            // Was not implemented, so try the other method.
                            let message = responses[0].clone();
                            datapoint_result = monitor.process_response(response.host.clone(), message.clone(), parent_datapoint.clone().unwrap_or_default())
                        }
                        else {
                            datapoint_result = Err(error);
                        }
                    }
                }
                else {
                    log::warn!("No response messages received for monitor {}", monitor_id);
                    // This is just ignored below.
                    datapoint_result = Err(String::new());
                }

                let new_data_point = match datapoint_result {
                    Ok(data_point) => {
                        log::debug!("[{}][{}] Data point received: {} {}", response.host.name, monitor_id, data_point.label, data_point);
                        data_point
                    },
                    Err(_) => {
                        // In case this was an extension module, retain the parents data point unmodified.
                        parent_datapoint.clone().unwrap_or_default()
                    }
                };

                for error in errors.iter() {
                    log::error!("[{}][{}] Error: {}", response.host.name, monitor_id, error.message);
                }

                if extension_monitors.len() > 0 {
                    // Process extension modules until the final result is reached.
                    let next_monitor_id = extension_monitors.remove(0);
                    let next_monitor = &monitors[&response.host.name][&next_monitor_id];
                    let next_parent_datapoint = parent_datapoint.unwrap_or_else(|| new_data_point.clone());

                    let messages = match get_monitor_connector_messages(&response.host, &next_monitor, &next_parent_datapoint) {
                        Ok(messages) => messages,
                        Err(error1) => {
                            log::error!("[{}][{}] Monitor failed: {}", response.host.name, monitor_id, error1);

                            if let Err(error2) = state_update_sender.send(StateUpdateMessage {
                                host_name: response.host.name.clone(),
                                display_options: next_monitor.get_display_options(),
                                module_spec: next_monitor.get_module_spec(),
                                errors: vec![error1],
                                invocation_id: response.invocation_id,
                                ..Default::default()
                            }) {
                                log::error!("Failed to send state update: {}", error2);
                                panic!("Failed to send state update: {}", error2);
                            }

                            return;
                        }
                    };

                    if let Err(error) = request_sender.send(ConnectorRequest {
                        connector_spec: next_monitor.get_connector_spec(),
                        source_id: next_monitor.get_module_spec().id,
                        host: response.host.clone(),
                        invocation_id: response.invocation_id,
                        response_sender: response_sender.clone(),
                        request_type: RequestType::MonitorCommand {
                            parent_datapoint: Some(new_data_point.clone()),
                            extension_monitors: extension_monitors,
                            commands: messages,
                        },
                    }) {
                        log::error!("[{}][{}] Failed to send connector request: {}", response.host.name, next_monitor_id, error);

                        if let Err(error) = state_update_sender.send(StateUpdateMessage::fatal_error()) {
                            log::error!("Failed to send state update: {}", error);
                            panic!("Failed to send state update: {}", error);
                        }
                    }
                }
                else {
                    if let Err(error) = state_update_sender.send(StateUpdateMessage {
                        host_name: response.host.name.clone(),
                        display_options: monitor.get_display_options(),
                        module_spec: monitor.get_module_spec(),
                        data_point: Some(new_data_point),
                        errors: errors,
                        invocation_id: response.invocation_id,
                        ..Default::default()
                    }) {
                        log::error!("Failed to send state update: {}", error);
                        panic!("Failed to send state update: {}", error);
                    }
                }
            }

            log::debug!("Stopping receiver thread");
        })
    }

}

/// NOTE: Panics are not handled gracefully since this runs in main UI thread.
/// get_connector_message and get_connector_messages should never panic.
fn get_monitor_connector_messages(host: &Host, monitor: &Monitor, parent_datapoint: &DataPoint) -> Result<Vec<String>, LkError> {
    let mut all_messages: Vec<String> = Vec::new();

    match monitor.get_connector_messages(host.clone(), parent_datapoint.clone()) {
        Ok(messages) => all_messages.extend(messages),
        Err(error) => {
            if error.kind != ErrorKind::NotImplemented {
                return Err(LkError::from(error).set_source(monitor.get_module_spec().id));
            }
        }
    }

    match monitor.get_connector_message(host.clone(), parent_datapoint.clone()) {
        Ok(message) => all_messages.push(message),
        Err(error) => {
            if error.kind != ErrorKind::NotImplemented {
                return Err(error.set_source(monitor.get_module_spec().id));
            }
        }
    }

    Ok(all_messages)
}
