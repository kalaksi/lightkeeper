/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::{
    collections::HashMap,
    sync::mpsc,
    sync::Arc,
    sync::Mutex,
    thread,
};

use serde_derive::{Deserialize, Serialize};

use crate::error::LkError;
use crate::module::monitoring::DataPoint;
use crate::monitor_manager::CERT_MONITOR_HOST_ID;
use crate::Host;
use crate::configuration::{ConfigGroup, HostSettings, Hosts};
use crate::file_handler::{self, FileMetadata};
use crate::module::{ModuleFactory, ModuleSpecification, ModuleType};
use crate::module::connection::*;

use self::request_response::RequestResponse;


type ConnectorStates = HashMap<ModuleSpecification, Connector>;


const MAX_WORKER_THREADS: usize = 6;


// Default needs to be implemented because of Qt QObject requirements.
#[derive(Default)]
pub struct ConnectionManager {
    /// Key is host name/id.
    stateful_connectors: Arc<Mutex<HashMap<String, ConnectorStates>>>,
    module_factory: Arc<ModuleFactory>,
    /// Only meant for tracking config changes in re-configuration.
    current_config: HashMap<String, ConfigGroup>,

    request_receiver: Option<mpsc::Receiver<ConnectorRequest>>,
    request_sender_prototype: Option<mpsc::Sender<ConnectorRequest>>,
    receiver_thread: Option<thread::JoinHandle<()>>,
}

impl ConnectionManager {
    pub fn new(module_factory: Arc<ModuleFactory>) -> Self {
        ConnectionManager {
            stateful_connectors: Arc::new(Mutex::new(HashMap::new())),
            module_factory: module_factory,
            ..Default::default()
        }
    }

    pub fn configure(&mut self, hosts_config: &Hosts) {
        self.stop();

        let mut stateful_connectors = self.stateful_connectors.lock().unwrap();

        let new_host_configs = if stateful_connectors.is_empty() {
            // For certificate monitoring.
            let cert_monitor_connectors = stateful_connectors.entry(CERT_MONITOR_HOST_ID.to_string()).or_insert(HashMap::new());
            let mut settings = HashMap::new();
            settings.insert("verify_certificate".to_string(), "true".to_string());
            let cert_monitor_connector = Tcp::new_connection_module(&settings);
            cert_monitor_connectors.insert(cert_monitor_connector.get_module_spec(), cert_monitor_connector);

            // All hosts.
            hosts_config.hosts.clone()
        }
        else {
            // Re-add hosts that had their config changed.
            for (host_id, new_host_config) in hosts_config.hosts.iter() {
                if let Some(current_host_config) = self.current_config.get(host_id) {
                    if current_host_config.connectors != new_host_config.effective.connectors {
                        stateful_connectors.remove(host_id);
                    }
                }
            }

            // Remove connectors for hosts that are no longer present.
            stateful_connectors.retain(|host_id, _|
                hosts_config.hosts.contains_key(host_id) || host_id == CERT_MONITOR_HOST_ID
            );

            hosts_config.hosts.clone().into_iter()
                .filter(|(host_id, _)| !stateful_connectors.contains_key(host_id))
                .collect::<BTreeMap<String, HostSettings>>()
        };

        // For regular host monitoring.
        for (host_id, host_config) in new_host_configs {
            let host_connectors = stateful_connectors.entry(host_id.clone()).or_insert(HashMap::new());

            for (monitor_id, monitor_config) in host_config.effective.monitors.iter() {
                let monitor_spec = ModuleSpecification::monitor(monitor_id.as_str(), monitor_config.version.as_str());
                let monitor = match self.module_factory.new_monitor(&monitor_spec, &monitor_config.settings) {
                    Some(monitor) => monitor,
                    None => continue,
                };

                if let Some(mut connector_spec) = monitor.get_connector_spec() {
                    connector_spec.module_type = ModuleType::Connector;

                    let connector_settings = match host_config.effective.connectors.get(&connector_spec.id) {
                        Some(config) => config.settings.clone(),
                        None => HashMap::new(),
                    };

                    let connector = match self.module_factory.new_connector(&connector_spec, &connector_settings) {
                        Some(connector) => connector,
                        None => continue,
                    };

                    if !connector.get_metadata_self().is_stateless {
                        host_connectors.entry(connector_spec).or_insert_with(|| connector);
                    }
                }
            }

            for (command_id, command_config) in host_config.effective.commands.iter() {
                let command_spec = ModuleSpecification::command(command_id, &command_config.version);
                let command = match self.module_factory.new_command(&command_spec, &command_config.settings) {
                    Some(command) => command,
                    None => continue,
                };

                if let Some(connector_spec) = command.get_connector_spec() {
                    let connector_settings = match host_config.effective.connectors.get(&connector_spec.id) {
                        Some(config) => config.settings.clone(),
                        None => HashMap::new(),
                    };

                    let connector = match self.module_factory.new_connector(&connector_spec, &connector_settings) {
                        Some(connector) => connector,
                        None => continue,
                    };

                    if !connector.get_metadata_self().is_stateless {
                        host_connectors.entry(connector_spec).or_insert_with(|| connector);
                    }
                }
            }
        }

        self.current_config = hosts_config.hosts.iter()
            .map(|(host_id, config)| (host_id.clone(), config.effective.clone()))
            .collect();

        let (sender, receiver) = mpsc::channel::<ConnectorRequest>();
        self.request_receiver = Some(receiver);
        self.request_sender_prototype = Some(sender);
    }

    pub fn new_request_sender(&mut self) -> mpsc::Sender<ConnectorRequest> {
        self.request_sender_prototype.as_ref().unwrap().clone()
    }

    pub fn start_processing_requests(&mut self) {
        let thread = Self::process_requests(
            self.stateful_connectors.clone(),
            self.request_receiver.take().unwrap(),
            self.module_factory.clone(),
        );
        self.receiver_thread = Some(thread);
    }

    pub fn stop(&mut self) {
        if let Some(thread) = self.receiver_thread.take() {
            self.new_request_sender()
                .send(ConnectorRequest::exit_token())
                .unwrap_or_else(|error| log::error!("Couldn't send exit token to connection manager: {}", error));

            if let Err(error) = thread.join() {
                log::error!("Error in thread: {:?}", error);
            }
        }
    }

    fn process_requests(
        stateful_connectors: Arc<Mutex<HashMap<String, ConnectorStates>>>,
        receiver: mpsc::Receiver<ConnectorRequest>,
        module_factory: Arc<ModuleFactory>) -> thread::JoinHandle<()> {

        thread::spawn(move || {
            let worker_pool = rayon::ThreadPoolBuilder::new().num_threads(MAX_WORKER_THREADS).build()
                .expect("Failed to create connection worker pool");

            log::debug!("Created worker pool with {} threads", MAX_WORKER_THREADS);

            loop {
                let stateful_connectors = stateful_connectors.clone();
                let module_factory = module_factory.clone();

                let request = match receiver.recv() {
                    Ok(data) => data,
                    Err(error) => {
                        log::error!("Stopped receiver thread: {}", error);
                        return;
                    }
                };

                if let RequestType::Exit = request.request_type {
                    log::debug!("Gracefully stopping request processing");
                    return;
                }

                let connector_spec = match &request.connector_spec {
                    Some(spec) => spec.clone(),
                    None => {
                        // Requests with no connector dependency.
                        if let Err(error) = request.response_sender.send(RequestResponse::new_empty(&request)) {
                            log::error!("Failed to send response: {}", error);
                            return;
                        }

                        continue;
                    }
                };

                worker_pool.spawn(move || {
                    log::debug!("[{}][{}] Worker {} processing a request",
                        request.host.name, request.source_id, rayon::current_thread_index().unwrap_or_default());

                    let connector_metadata = module_factory.get_connector_module_metadata(&connector_spec);

                    let connector = {
                        let stateful_connectors = stateful_connectors.lock().unwrap();

                        // Stateless connectors.
                        if connector_metadata.is_stateless {
                            match module_factory.new_connector(&connector_spec, &HashMap::new()) {
                                Some(connector) => connector,
                                None => return,
                            }
                        }
                        // Stateful connectors.
                        else {
                            match stateful_connectors.get(&request.host.name)
                                .and_then(|host_connectors| host_connectors.get(&connector_spec))
                            {
                                Some(connector) => connector.box_clone(),
                                None => {
                                    log::error!("[{}][{}] host connection is not configured", request.host.name, request.source_id);
                                    return;
                                }
                            }
                        }
                    };

                    connector.set_target(&request.host.get_address());

                    // Key verifications have to be done before anything else.
                    match request.request_type {
                        RequestType::KeyVerification { ref key_id } => {
                            log::debug!("[{}] Verifying host key", request.host.name);

                            if let Err(error) = connector.verify_host_key(&request.host.get_address(), &key_id) {
                                let response = RequestResponse::new(
                                    &request,
                                    vec![Err(error.set_source(connector.get_module_spec().id))],
                                );

                                if let Err(error) = request.response_sender.send(response) {
                                    log::error!("Failed to send response: {}", error);
                                }
                            }

                            return;
                        },
                        _ => {}
                    }

                    let responses = match &request.request_type {
                        RequestType::MonitorCommand { extension_monitors: _, parent_datapoint: _, commands } => {
                            Self::process_commands(&request, &connector, &commands)
                        },
                        RequestType::Command { commands } => {
                            Self::process_commands(&request, &connector, &commands)
                        },
                        RequestType::CommandFollowOutput { commands } => {
                            if let [command] = &commands[..] {
                                vec![Self::process_command_follow_output(&request, &connector, command, request.response_sender.clone())]
                            }
                            else {
                                vec![Err(LkError::other("Follow output is only supported for a single command"))]
                            }
                        },
                        RequestType::Download { remote_file_path: file_path } =>
                            vec![Self::process_download(&request.host, &connector, &file_path)],
                        RequestType::Upload { metadata: _, local_file_path } =>
                            vec![Self::process_upload(&request.host, &connector, &local_file_path)],
                        _ => {
                            log::error!("[{}][{}] Unsupported request type", request.host.name, request.source_id);
                            vec![Err(LkError::other("Unsupported request type"))]
                        }
                    };

                    let response = RequestResponse::new(&request, responses);
                    if let Err(_) = request.response_sender.send(response) {
                        log::warn!("[{}][{}] Couldn't send response", request.host.name, request.source_id);
                    }
                });
            }
        })
    }

    fn process_commands(request: &ConnectorRequest,
                        connector: &Connector,
                        request_messages: &Vec<String>) -> Vec<Result<ResponseMessage, LkError>> {

        let mut results = Vec::new();
        for request_message in request_messages {
            // Some commands are supposed to not actually execute.
            if request_message.is_empty() {
                log::debug!("[{}][{}] Ignoring empty command", request.host.name, request.source_id);
                results.push(Ok(ResponseMessage::empty()));
            }
            else {
                log::debug!("[{}][{}] Command: {}", request.host.name, request.source_id, request_message);
            }

            let response_result = connector.send_message(request_message);

            if let Ok(response) = response_result {
                if response.return_code != 0 {
                    log::warn!("[{}][{}] Command returned non-zero exit code: {}",
                        request.host.name, request.source_id, response.return_code);
                }
                results.push(Ok(response))
            }
            else {
                // Add module name to error details.
                results.push(response_result.map_err(|error| error.set_source(connector.get_module_spec().id)));

                // Abort on any errors.
                break;
            }
        }

        results
    }

    fn process_command_follow_output(
        request: &ConnectorRequest,
        connector: &Connector,
        request_message: &String,
        response_sender: mpsc::Sender<RequestResponse>,
    ) -> Result<ResponseMessage, LkError> {

        log::debug!("[{}][{}] Command: {}", request.host.name, request.source_id, request_message);
        let mut response_message_result = connector.send_message_partial(request_message, request.invocation_id);

        // Paradoxical name...
        let mut full_partial_message = String::new();

        loop {
            // Wait for some time. No sense in processing too fast.
            thread::sleep(std::time::Duration::from_millis(100));

            match response_message_result {
                Ok(mut response_message) => {
                    full_partial_message.push_str(&response_message.message);
                    response_message.message = full_partial_message.clone();

                    if response_message.is_partial {
                        let response = RequestResponse::new(request, vec![Ok(response_message)]);
                        if let Err(error) = response_sender.send(response) {
                            log::error!("Failed to send response: {}", error);
                        }

                        response_message_result = connector.receive_partial_response(request.invocation_id);
                    }
                    else {
                        if response_message.return_code != 0 {
                            log::warn!("[{}][{}] Command returned non-zero exit code: {}",
                                request.host.name, request.source_id, response_message.return_code)
                        }

                        break Ok(response_message);
                    }
                },
                Err(ref error) => {
                    log::error!("[{}][{}] Error while receiving partial response: {}",
                        request.host.name, request.source_id, error);

                    break response_message_result;
                }
            }
        }
    }

    fn process_download(host: &Host, connector: &Connector, file_path: &str) -> Result<ResponseMessage, LkError> {
        log::debug!("[{}] Downloading file: {}", host.name, file_path);
        match connector.download_file(file_path) {
            Ok((metadata, contents)) => {
                match file_handler::create_file(host, file_path, metadata, contents) {
                    Ok(file_path) => Ok(ResponseMessage::new_success(file_path)),
                    Err(error) => Err(error.into()),
                }
            },
            Err(error) => Err(error),
        }
    }

    fn process_upload(host: &Host, connector: &Connector, local_file_path: &str) -> Result<ResponseMessage, LkError> {
        log::debug!("[{}] Uploading file: {}", host.name, local_file_path);
        match file_handler::read_file(local_file_path) {
            Ok((metadata, contents)) => {
                let result = connector.upload_file(&metadata, contents);

                // Returns empty or error as is.
                result.map(|_| ResponseMessage::empty())
            },
            Err(error) => Err(error.into()),
        }
    }
}

pub struct ConnectorRequest {
    pub connector_spec: Option<ModuleSpecification>,
    pub source_id: String,
    pub host: Host,
    pub invocation_id: u64,
    pub request_type: RequestType,
    pub response_sender: mpsc::Sender<RequestResponse>,
}

impl ConnectorRequest {
    pub fn exit_token() -> Self {
        let dummy_sender = mpsc::channel::<RequestResponse>().0;
        ConnectorRequest {
            connector_spec: None,
            source_id: String::new(),
            host: Host::default(),
            invocation_id: 0,
            request_type: RequestType::Exit,
            response_sender: dummy_sender,
        }
    }
}

impl Debug for ConnectorRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ConnectorRequest {:?}]", self.connector_spec)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum RequestType {
    MonitorCommand {
        extension_monitors: Vec<String>,
        parent_datapoint: Option<DataPoint>,
        commands: Vec<String>,
    },
    Command {
        commands: Vec<String>,
    },
    CommandFollowOutput {
        commands: Vec<String>,
    },
    Download {
        remote_file_path: String,
    },
    Upload {
        local_file_path: String,
        metadata: FileMetadata,
    },
    KeyVerification {
        key_id: String,
    },
    /// Causes the receiver thread to exit.
    #[default]
    Exit,
}