use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use crate::module::connection::RequestResponse;
use crate::Host;
use crate::configuration::{CacheSettings, Hosts};
use crate::enums::Criticality;
use crate::module::connection::ResponseMessage;
use crate::module::{monitoring::*, ModuleSpecification};
use crate::module::ModuleFactory;
use crate::host_manager::{StateUpdateMessage, HostManager};
use crate::connection_manager::{ ConnectorRequest, RequestType, CachePolicy };
use crate::utils::ErrorMessage;


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
    cache_settings: CacheSettings,

    // Shared resources.
    host_manager: Rc<RefCell<HostManager>>,
    module_factory: Arc<ModuleFactory>,

    response_sender_prototype: Option<mpsc::Sender<RequestResponse>>,
    response_receiver: Option<mpsc::Receiver<RequestResponse>>,
    response_receiver_thread: Option<thread::JoinHandle<()>>,
}

impl MonitorManager {
    pub fn new(cache_settings: CacheSettings,
               host_manager: Rc<RefCell<HostManager>>,
               module_factory: Arc<ModuleFactory>) -> Self {

        MonitorManager {
            cache_settings: cache_settings,
            host_manager: host_manager.clone(),
            module_factory: module_factory,
            ..Default::default()
        }
    }

    pub fn configure(&mut self,
                     hosts_config: &Hosts,
                     request_sender: mpsc::Sender<ConnectorRequest>,
                     state_update_sender: mpsc::Sender<StateUpdateMessage>) {

        self.monitors.lock().unwrap().clear();
        self.request_sender = Some(request_sender);
        self.state_update_sender = Some(state_update_sender);

        for (host_id, host_config) in hosts_config.hosts.iter() {

            let mut new_monitors = Vec::<Monitor>::new();
            for (monitor_id, monitor_config) in host_config.monitors.iter() {
                let monitor_spec = ModuleSpecification::new(monitor_id.as_str(), monitor_config.version.as_str());
                let monitor = self.module_factory.new_monitor(&monitor_spec, &monitor_config.settings);
                new_monitors.push(monitor);
            }

            let base_modules = new_monitors.iter().filter(|monitor| monitor.get_metadata_self().parent_module.is_some())
                                                  .map(|monitor| monitor.get_metadata_self().parent_module.unwrap())
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
        self.new_response_sender()
            .send(RequestResponse::stop())
            .unwrap_or_else(|error| log::error!("Couldn't send exit token to command handler: {}", error));

        if let Some(thread) = self.response_receiver_thread.take() {
            thread.join().unwrap();
        }
    }

    pub fn new_response_sender(&self) -> mpsc::Sender<RequestResponse> {
        self.response_sender_prototype.clone().unwrap()
    }

    // Adds a monitor but only if a monitor with the same ID doesn't exist.
    fn add_monitor(&mut self, host_id: String, monitor: Monitor, send_initial_value: bool) {
        let mut monitors = self.monitors.lock().unwrap();
        let monitor_collection = monitors.entry(host_id.clone()).or_insert(HashMap::new());
        let module_spec = monitor.get_module_spec();

        // Only add if missing.
        if !monitor_collection.contains_key(&module_spec.id) {

            if send_initial_value {
                // Add initial state value indicating no data as been received yet.
                self.state_update_sender.as_ref().unwrap().send(StateUpdateMessage {
                    host_name: host_id,
                    display_options: monitor.get_display_options(),
                    module_spec: monitor.get_module_spec(),
                    data_point: Some(DataPoint::no_data()),
                    ..Default::default()
                }).unwrap();
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
    pub fn refresh_platform_info_all(&mut self, cache_policy: Option<CachePolicy>) -> Vec<String> {
        let cache_policy = if let Some(cache_policy) = cache_policy {
            cache_policy
        }
        else if !self.cache_settings.enable_cache {
            CachePolicy::BypassCache
        }
        else if self.cache_settings.provide_initial_value {
            CachePolicy::PreferCache
        }
        else {
            CachePolicy::BypassCache
        };
        let host_ids = self.monitors.lock().unwrap().iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();
        for host_id in &host_ids {
            self.refresh_platform_info(host_id, Some(cache_policy));
        }

        host_ids
    }

    /// Refreshes platform info and such in preparation for actual monitor refresh.
    pub fn refresh_platform_info(&mut self, host_id: &String, cache_policy: Option<CachePolicy>) {
        let platform_info_providers = self.platform_info_providers.lock().unwrap();
        let monitors = self.monitors.lock().unwrap();
        let monitors_for_host = monitors.iter().filter(|(name, _)| &host_id == name);

        let cache_policy = if let Some(cache_policy) = cache_policy {
            cache_policy
        }
        else if !self.cache_settings.enable_cache {
            CachePolicy::BypassCache
        }
        else if self.cache_settings.provide_initial_value {
            CachePolicy::PreferCache
        }
        else {
            CachePolicy::BypassCache
        };

        for (host_name, monitor_collection) in monitors_for_host {
            let host = self.host_manager.borrow().get_host(host_name);

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
                        log::error!("Monitor \"{}\" failed: {}", info_provider.get_module_spec().id, error);
                        return;
                    }
                };

                self.invocation_id_counter += 1;

                // Notify host state manager about new pending monitor invocation.
                self.state_update_sender.as_ref().unwrap().send(StateUpdateMessage {
                    host_name: host.name.clone(),
                    display_options: info_provider.get_display_options(),
                    module_spec: info_provider.get_module_spec(),
                    data_point: Some(DataPoint::pending(self.invocation_id_counter)),
                    ..Default::default()
                }).unwrap();

                self.request_sender.as_ref().unwrap().send(ConnectorRequest {
                    connector_spec: info_provider.get_connector_spec(),
                    source_id: info_provider.get_module_spec().id,
                    host: host.clone(),
                    invocation_id: self.invocation_id_counter,
                    response_sender: self.new_response_sender(),
                    request_type: RequestType::MonitorCommand {
                        parent_datapoint: None,
                        extension_monitors: Vec::new(),
                        cache_policy: cache_policy,
                        commands: commands,
                    },
                }).unwrap();
            }
        }
    }

    pub fn get_all_host_categories(&self, host_id: &String) -> Vec<String> {
        let mut categories = self.monitors.lock().unwrap()[host_id].iter()
                                          .map(|(_, monitor)| monitor.get_display_options().category.clone())
                                          .collect::<Vec<_>>();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Returns the invocation IDs of the refresh operations.
    pub fn refresh_monitors_of_category_control(&mut self, host_id: &String, category: &String, cache_policy: CachePolicy) -> Vec<u64> {
        let host = self.host_manager.borrow().get_host(host_id);
        let monitors = self.monitors.lock().unwrap();
        let monitors_by_category = monitors[host_id].iter()
                                                    .filter(|(_, monitor)| &monitor.get_display_options().category == category)
                                                    .collect();

        let invocation_ids = self.refresh_monitors(host, monitors_by_category, cache_policy);
        self.invocation_id_counter += invocation_ids.len() as u64;
        invocation_ids
    }

    /// Returns the invocation IDs of the refresh operations.
    pub fn refresh_monitors_of_category(&mut self, host_id: &String, category: &String) -> Vec<u64> {
        let host = self.host_manager.borrow().get_host(host_id);
        let monitors = self.monitors.lock().unwrap();
        let monitors_by_category = monitors[host_id].iter()
                                                    .filter(|(_, monitor)| &monitor.get_display_options().category == category)
                                                    .collect();

        let cache_policy = if !self.cache_settings.enable_cache {
            CachePolicy::BypassCache
        }
        else if self.cache_settings.prefer_cache {
            CachePolicy::PreferCache
        }
        else {
            CachePolicy::BypassCache
        };

        let invocation_ids = self.refresh_monitors(host, monitors_by_category, cache_policy);
        self.invocation_id_counter += invocation_ids.len() as u64;
        invocation_ids
    }

    /// Refresh by monitor ID.
    /// Returns the invocation IDs of the refresh operations.
    pub fn refresh_monitors_by_id(&mut self, host_id: &String, monitor_id: &String, cache_policy: CachePolicy) -> Vec<u64> {
        let host = self.host_manager.borrow().get_host(host_id);
        let monitors = self.monitors.lock().unwrap();
        let monitor = monitors[host_id].iter()
                                       .filter(|(_, monitor)| &monitor.get_module_spec().id == monitor_id)
                                       .collect();

        let invocation_ids = self.refresh_monitors(host, monitor, cache_policy);
        self.invocation_id_counter += invocation_ids.len() as u64;
        invocation_ids
    }

    fn refresh_monitors(&self, host: Host, monitors: HashMap<&String, &Monitor>, cache_policy: CachePolicy) -> Vec<u64> {
        if !host.platform.is_set() {
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
            self.state_update_sender.as_ref().unwrap().send(StateUpdateMessage {
                host_name: host.name.clone(),
                display_options: monitor.get_display_options(),
                module_spec: monitor.get_module_spec(),
                data_point: Some(DataPoint::pending(current_invocation_id)),
                ..Default::default()
            }).unwrap();

            let messages = match get_monitor_connector_messages(&host, &monitor, &DataPoint::empty()) {
                Ok(messages) => messages,
                Err(error) => {
                    log::error!("Monitor \"{}\" failed: {}", monitor.get_module_spec().id, error);
                    let ui_error = format!("{}: {}", monitor.get_module_spec().id, error);

                    self.state_update_sender.as_ref().unwrap().send(StateUpdateMessage {
                        host_name: host.name.clone(),
                        display_options: monitor.get_display_options(),
                        module_spec: monitor.get_module_spec(),
                        errors: vec![ErrorMessage::new(Criticality::Error, ui_error)],
                        ..Default::default()
                    }).unwrap();

                    continue;
                }
            };

            self.request_sender.as_ref().unwrap().send(ConnectorRequest {
                connector_spec: monitor.get_connector_spec(),
                source_id: monitor.get_module_spec().id,
                host: host.clone(),
                invocation_id: current_invocation_id,
                response_sender: self.new_response_sender(),
                request_type: RequestType::MonitorCommand {
                    parent_datapoint: None,
                    extension_monitors: extension_ids,
                    cache_policy: cache_policy,
                    commands: messages,
                },
            }).unwrap();
        }

        invocation_ids
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
            loop {
                let response = match response_receiver.recv() {
                    Ok(response) => response,
                    Err(error) => {
                        log::error!("Stopped response receiver thread: {}", error);
                        return;
                    }
                };

                if response.stop {
                    log::debug!("Gracefully stopping receiver thread");
                    return;
                }

                let results_len = response.responses.len();
                let (responses, errors): (Vec<_>, Vec<_>) =  response.responses.into_iter().partition(Result::is_ok);
                let responses = responses.into_iter().map(Result::unwrap).collect::<Vec<_>>();
                let mut errors = errors.into_iter().map(|error| ErrorMessage::new(Criticality::Error, error.unwrap_err())).collect::<Vec<_>>();

                // If CachePolicy::OnlyCache is used and an entry is not found, don't continue.
                if responses.iter().any(|response| response.is_not_found()) {
                    return;
                }

                let monitors = monitors.lock().unwrap();
                let platform_info_providers = platform_info_providers.lock().unwrap();
                let monitor_id = &response.source_id;
                // Search from internal monitors first.
                let monitor = platform_info_providers.get(monitor_id).unwrap_or_else(|| {
                    &monitors[&response.host.name][monitor_id]
                });

                // Have to be done separately because of name conflict.
                let parent_datapoint = match response.request_type.clone() {
                    RequestType::MonitorCommand { parent_datapoint, .. } => parent_datapoint.clone().unwrap_or_default(),
                    _ => panic!("Invalid request type: {:?}", response.request_type)
                };

                let mut datapoint_result;
                if results_len == 0 {
                    // Some special modules require no connectors and receive no response messages, such
                    // as `os`, which only uses existing platform info.
                    datapoint_result = monitor.process_response(response.host.clone(), ResponseMessage::empty(), parent_datapoint.clone())
                }
                else if responses.len() > 0 {
                    datapoint_result = monitor.process_responses(response.host.clone(), responses.clone(), parent_datapoint.clone());
                    if let Err(error) = datapoint_result {
                        if error.is_empty() {
                            // Was not implemented, so try the other method.
                            let message = responses[0].clone();
                            datapoint_result = monitor.process_response(response.host.clone(), message.clone(), parent_datapoint.clone())
                                                        .map(|mut data_point| { data_point.is_from_cache = message.is_from_cache; data_point });
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
                    Ok(mut data_point) => {
                        log::debug!("[{}] Data point received for monitor {}: {} {}", response.host.name, monitor_id, data_point.label, data_point);
                        data_point.invocation_id = response.invocation_id;
                        data_point
                    },
                    Err(error) => {
                        if !error.is_empty() {
                            errors.push(ErrorMessage::new(Criticality::Error, error));
                        }
                        // In case this was an extension module, retain the parents data point unmodified.
                        parent_datapoint.clone()
                    }
                };

                for error in errors.iter() {
                    log::error!("[{}] Error from monitor {}: {}", response.host.name, monitor_id, error.message);
                }

                let cache_policy = match response.request_type {
                    RequestType::MonitorCommand { cache_policy, .. } => cache_policy,
                    _ => panic!()
                };
                let mut extension_monitors = match response.request_type {
                    RequestType::MonitorCommand { extension_monitors, .. } => extension_monitors,
                    _ => panic!()
                };

                if extension_monitors.len() > 0 {
                    // Process extension modules until the final result is reached.
                    let next_monitor_id = extension_monitors.remove(0);
                    let next_monitor = &monitors[&response.host.name][&next_monitor_id];

                    let messages = match get_monitor_connector_messages(&response.host, &monitor, &parent_datapoint) {
                        Ok(messages) => messages,
                        Err(error) => {
                            log::error!("Monitor \"{}\" failed: {}", monitor.get_module_spec().id, error);
                            let ui_error = format!("{}: {}", monitor.get_module_spec().id, error);

                            state_update_sender.send(StateUpdateMessage {
                                host_name: response.host.name.clone(),
                                display_options: monitor.get_display_options(),
                                module_spec: monitor.get_module_spec(),
                                errors: vec![ErrorMessage::new(Criticality::Error, ui_error)],
                                ..Default::default()
                            }).unwrap();

                            return;
                        }
                    };

                    request_sender.send(ConnectorRequest {
                        connector_spec: next_monitor.get_connector_spec(),
                        source_id: next_monitor.get_module_spec().id,
                        host: response.host.clone(),
                        invocation_id: response.invocation_id,
                        response_sender: response_sender.clone(),
                        request_type: RequestType::MonitorCommand {
                            parent_datapoint: Some(parent_datapoint.clone()),
                            extension_monitors: extension_monitors,
                            cache_policy: cache_policy,
                            commands: messages,
                        },
                    }).unwrap();
                }
                else {
                    state_update_sender.send(StateUpdateMessage {
                        host_name: response.host.name.clone(),
                        display_options: monitor.get_display_options(),
                        module_spec: monitor.get_module_spec(),
                        data_point: Some(new_data_point),
                        errors: errors,
                        ..Default::default()
                    }).unwrap();
                }

            }
        })
    }

}

fn get_monitor_connector_messages(host: &Host, monitor: &Monitor, parent_datapoint: &DataPoint) -> Result<Vec<String>, String> {
    let mut all_messages: Vec<String> = Vec::new();

    match monitor.get_connector_messages(host.clone(), parent_datapoint.clone()) {
        Ok(messages) => all_messages.extend(messages),
        Err(error) => {
            if !error.is_empty() {
                return Err(error);
            }
        }
    }

    match monitor.get_connector_message(host.clone(), parent_datapoint.clone()) {
        Ok(message) => all_messages.push(message),
        Err(error) => {
            if !error.is_empty() {
                return Err(error);
            }
        }
    }

    Ok(all_messages)
}
