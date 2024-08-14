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
use crate::configuration::{CacheSettings, Hosts};
use crate::file_handler::{self, FileMetadata};
use crate::module::{ModuleFactory, ModuleSpecification, ModuleType};
use crate::module::connection::*;
use crate::cache::{Cache, CacheScope};

use self::request_response::RequestResponse;


type ConnectorStates = HashMap<ModuleSpecification, Connector>;


const MAX_WORKER_THREADS: usize = 4;


// Default needs to be implemented because of Qt QObject requirements.
#[derive(Default)]
pub struct ConnectionManager {
    /// Key is host name/id.
    stateful_connectors: Option<HashMap<String, ConnectorStates>>,
    module_factory: Arc<ModuleFactory>,
    cache_settings: CacheSettings,

    request_receiver: Option<mpsc::Receiver<ConnectorRequest>>,
    request_sender_prototype: Option<mpsc::Sender<ConnectorRequest>>,
    receiver_thread: Option<thread::JoinHandle<()>>,
}

impl ConnectionManager {
    pub fn new(module_factory: Arc<ModuleFactory>) -> Self {
        ConnectionManager {
            stateful_connectors: Some(HashMap::new()),
            module_factory: module_factory,
            ..Default::default()
        }
    }

    pub fn configure(&mut self, hosts_config: &Hosts, cache_settings: &CacheSettings) {
        self.stop();

        self.stateful_connectors = Some(HashMap::new());
        self.cache_settings = cache_settings.clone();
        let stateful_connectors = self.stateful_connectors.as_mut().unwrap();

        // For certificate monitoring.
        let cert_monitor_connectors = stateful_connectors.entry(CERT_MONITOR_HOST_ID.to_string()).or_insert(HashMap::new());
        let mut settings = HashMap::new();
        settings.insert("verify_certificate".to_string(), "true".to_string());
        let cert_monitor_connector = Tcp::new_connection_module(&settings);
        cert_monitor_connectors.insert(cert_monitor_connector.get_module_spec(), cert_monitor_connector);

        // For regular host monitoring.
        for (host_id, host_config) in hosts_config.hosts.iter() {
            stateful_connectors.entry(host_id.clone()).or_insert(HashMap::new());
            let host_connectors = stateful_connectors.get_mut(host_id).unwrap();

            for (monitor_id, monitor_config) in host_config.effective.monitors.iter() {
                let monitor_spec = ModuleSpecification::monitor(monitor_id.as_str(), monitor_config.version.as_str());
                let monitor = self.module_factory.new_monitor(&monitor_spec, &monitor_config.settings);

                if let Some(mut connector_spec) = monitor.get_connector_spec() {
                    connector_spec.module_type = ModuleType::Connector;

                    let connector_settings = match host_config.effective.connectors.get(&connector_spec.id) {
                        Some(config) => config.settings.clone(),
                        None => HashMap::new(),
                    };

                    let connector = self.module_factory.new_connector(&connector_spec, &connector_settings);
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

                    let connector = self.module_factory.new_connector(&connector_spec, &connector_settings);
                    if !connector.get_metadata_self().is_stateless {
                        host_connectors.entry(connector_spec).or_insert_with(|| connector);
                    }
                }
            }
        }

        let (sender, receiver) = mpsc::channel::<ConnectorRequest>();
        self.request_receiver = Some(receiver);
        self.request_sender_prototype = Some(sender);
    }

    pub fn new_request_sender(&mut self) -> mpsc::Sender<ConnectorRequest> {
        self.request_sender_prototype.as_ref().unwrap().clone()
    }

    pub fn start_processing_requests(&mut self) {
        let thread = Self::process_requests(
            self.stateful_connectors.take().unwrap(),
            self.request_receiver.take().unwrap(),
            self.module_factory.clone(),
            self.cache_settings.clone()
        );
        self.receiver_thread = Some(thread);
    }

    pub fn stop(&mut self) {
        if let Some(thread) = self.receiver_thread.take() {
            self.new_request_sender()
                .send(ConnectorRequest::exit_token())
                .unwrap_or_else(|error| log::error!("Couldn't send exit token to connection manager: {}", error));

            thread.join().unwrap();
        }
    }

    fn initialize_cache(cache_settings: CacheSettings) -> Cache<String, ResponseMessage> {
        let mut new_command_cache = Cache::<String, ResponseMessage>::new(cache_settings.time_to_live, cache_settings.initial_value_time_to_live);

        if cache_settings.enable_cache {
            match new_command_cache.read_from_disk() {
                Ok(count) => log::debug!("Added {} entries from cache file", count),
                // Failing to read cache file is not critical.
                Err(error) => log::error!("{}", error),
            }
            log::debug!("Initialized cache with TTL of {} ({}) seconds", cache_settings.time_to_live, cache_settings.initial_value_time_to_live);
        }
        else {
            log::debug!("Cache is disabled. Clearing existing cache files.");
            // Clear any existing cache entries from cache file.
            new_command_cache.purge();
        }

        new_command_cache
    }

    fn process_requests(
        stateful_connectors: HashMap<String, ConnectorStates>,
        receiver: mpsc::Receiver<ConnectorRequest>,
        module_factory: Arc<ModuleFactory>,
        cache_settings: CacheSettings) -> thread::JoinHandle<()> {

        thread::spawn(move || {
            let worker_pool = rayon::ThreadPoolBuilder::new().num_threads(MAX_WORKER_THREADS).build().unwrap();
            log::debug!("Created worker pool with {} threads", MAX_WORKER_THREADS);

            let original_command_cache = Arc::new(Mutex::new(Self::initialize_cache(cache_settings.clone())));
            let stateful_connectors_arc = Arc::new(stateful_connectors);

            loop {
                let command_cache = original_command_cache.clone();
                let module_factory = module_factory.clone();
                let stateful_connectors = stateful_connectors_arc.clone();

                let request = match receiver.recv() {
                    Ok(data) => data,
                    Err(error) => {
                        log::error!("Stopped receiver thread: {}", error);
                        return;
                    }
                };

                if let RequestType::Exit = request.request_type {
                    log::debug!("Gracefully stopping connection manager thread");
                    

                    if cache_settings.enable_cache {
                        match &command_cache.lock().unwrap().write_to_disk() {
                            Ok(count) => log::debug!("Wrote {} entries to cache file", count),
                            // Failing to write the file is not critical.
                            Err(error) => log::error!("{}", error),
                        }
                    }
                    return;
                }

                let connector_spec = match &request.connector_spec {
                    Some(spec) => spec.clone(),
                    None => {
                        // Requests with no connector dependency.
                        request.response_sender.send(RequestResponse::new_empty(&request)).unwrap();
                        continue;
                    }
                };

                worker_pool.spawn(move || {
                    log::debug!("[{}][{}] Worker {} processing a request",
                        request.host.name, request.source_id, rayon::current_thread_index().unwrap_or_default());

                    let connector_metadata = module_factory.get_connector_module_metadata(&connector_spec);

                    // Stateless connectors.
                    let mut stateless_connector;
                    let connector = if connector_metadata.is_stateless {
                        stateless_connector = Box::new(module_factory.new_connector(&connector_spec, &HashMap::new()));
                        &mut stateless_connector
                    }
                    // Stateful connectors.
                    else {
                        let host_connectors = stateful_connectors.get(&request.host.name).unwrap();
                        host_connectors.get(&connector_spec).unwrap()
                    };

                    // TODO: not very elegant. No need to set multiple times.
                    connector.set_target(&request.host.get_address());

                    // Key verifications have to be done before anything else.
                    match request.request_type {
                        RequestType::KeyVerification { key_id } => {
                            log::debug!("[{}] Verifying host key", request.host.name);
                            connector.verify_host_key(&request.host.get_address(), &key_id).unwrap();
                            return;
                        },
                        _ => {}
                    }

                    let responses = match &request.request_type {
                        RequestType::MonitorCommand { cache_policy, extension_monitors: _, parent_datapoint: _, commands } => {
                            Self::process_commands(&request, command_cache.clone(), &cache_policy, &connector, &commands)
                        },
                        RequestType::Command { commands } => {
                            Self::process_commands(&request, command_cache.clone(), &CachePolicy::BypassCache, &connector, &commands)
                        },
                        RequestType::CommandFollowOutput { commands } => {
                            if commands.len() != 1 {
                                vec![Err(LkError::other("Follow output is only supported for a single command"))]
                            }
                            else {
                                let command = commands.first().unwrap();
                                vec![Self::process_command_follow_output(&request, &connector, command, request.response_sender.clone())]
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
                    request.response_sender.send(response).unwrap_or_else(|_response|
                        log::warn!("[{}][{}] Couldn't process response", request.host.name, request.source_id)
                    );
                });
            }
        })
    }

    fn process_commands(request: &ConnectorRequest,
                        command_cache: Arc<Mutex<Cache<String, ResponseMessage>>>,
                        cache_policy: &CachePolicy,
                        connector: &Connector,
                        request_messages: &Vec<String>) -> Vec<Result<ResponseMessage, LkError>> {


        // let request = request.lock().unwrap();
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

            let cache_key = match connector.get_metadata_self().cache_scope {
                CacheScope::Global => format!("{}|{}", connector.get_module_spec(), request_message),
                CacheScope::Host => format!("{}|{}|{}", request.host.name, connector.get_module_spec(), request_message),
            };

            let cached_response = if *cache_policy == CachePolicy::OnlyCache || *cache_policy == CachePolicy::PreferCache {
                let mut command_cache = command_cache.lock().unwrap();
                command_cache.get(&cache_key)
            }
            else {
                None
            };

            if let Some(cached_response) = cached_response {
                log::debug!("[{}][{}] Using cached response for command {}", request.host.name, request.source_id, request_message);
                results.push(Ok(cached_response));
            }
            else if *cache_policy == CachePolicy::OnlyCache {
                results.push(Ok(ResponseMessage::not_found()));
            }
            else {
                let response_result = connector.send_message(request_message);

                if let Ok(response) = response_result {
                    if response.return_code != 0 {
                        log::warn!("[{}][{}] Command returned non-zero exit code: {}",
                            request.host.name, request.source_id, response.return_code);
                    }
                    else {
                        if *cache_policy != CachePolicy::BypassCache {
                            // Doesn't cache failed commands.
                            let mut cached_response = response.clone();
                            cached_response.is_from_cache = true;
                            let mut command_cache = command_cache.lock().unwrap();
                            command_cache.insert(cache_key, cached_response);
                        }
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
            if let Ok(mut response_message) = response_message_result {
                full_partial_message.push_str(&response_message.message);
                response_message.message = full_partial_message.clone();

                if response_message.is_partial {
                    let response = RequestResponse::new(request, vec![Ok(response_message)]);
                    response_sender.send(response).unwrap();

                    response_message_result = connector.receive_partial_response(request.invocation_id);
                }
                else {
                    if response_message.return_code != 0 {
                        log::warn!("[{}][{}] Command returned non-zero exit code: {}",
                            request.host.name, request.source_id, response_message.return_code)
                    }

                    break Ok(response_message);
                }
            }
            else {
                log::error!("[{}][{}] Error while receiving partial response: {}",
                    request.host.name, request.source_id, response_message_result.clone().err().unwrap());
                break response_message_result;
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
        cache_policy: CachePolicy,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CachePolicy {
    #[default]
    BypassCache,
    PreferCache,
    OnlyCache,
}