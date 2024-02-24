use std::fmt::Debug;
use std::{
    collections::HashMap,
    sync::mpsc,
    sync::Arc,
    sync::Mutex,
    thread,
};

use rayon::prelude::*;

use crate::Host;
use crate::configuration::{CacheSettings, Hosts};
use crate::file_handler::{self, FileMetadata};
use crate::module::ModuleFactory;
use crate::module::ModuleSpecification;
use crate::module::connection::*;
use crate::cache::{Cache, CacheScope};

use self::request_response::RequestResponse;

pub type ResponseHandlerCallback = Box<dyn FnOnce(RequestResponse) + Send + 'static>;
type ConnectorStates = HashMap<ModuleSpecification, Arc<Mutex<Connector>>>;


const MAX_WORKER_THREADS: usize = 8;


// Default needs to be implemented because of Qt QObject requirements.
#[derive(Default)]
pub struct ConnectionManager {
    stateful_connectors: Option<HashMap<String, ConnectorStates>>,
    request_receiver: Option<mpsc::Receiver<ConnectorRequest>>,
    request_sender_prototype: Option<mpsc::Sender<ConnectorRequest>>,
    receiver_thread: Option<thread::JoinHandle<()>>,
    cache_settings: CacheSettings,

    // Shared resources.
    module_factory: Arc<ModuleFactory>,
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
        if self.receiver_thread.is_some() {
            self.stop();
        }

        self.stateful_connectors = Some(HashMap::new());
        self.cache_settings = cache_settings.clone();
        let stateful_connectors = self.stateful_connectors.as_mut().unwrap();

        for (host_id, host_config) in hosts_config.hosts.iter() {
            stateful_connectors.entry(host_id.clone()).or_insert(HashMap::new());
            let host_connectors = stateful_connectors.get_mut(host_id).unwrap();

            for (monitor_id, monitor_config) in host_config.monitors.iter() {
                let monitor_spec = ModuleSpecification::new(monitor_id.as_str(), monitor_config.version.as_str());
                let monitor = self.module_factory.new_monitor(&monitor_spec, &monitor_config.settings);

                if let Some(mut connector_spec) = monitor.get_connector_spec() {
                    connector_spec.module_type = String::from("connector");

                    let connector_settings = match host_config.connectors.get(&connector_spec.id) {
                        Some(config) => config.settings.clone(),
                        None => HashMap::new(),
                    };

                    let connector = self.module_factory.new_connector(&connector_spec, &connector_settings);
                    if !connector.get_metadata_self().is_stateless {
                        host_connectors.entry(connector_spec).or_insert_with(|| Arc::new(Mutex::new(connector)));
                    }
                }
            }

            for (command_id, command_config) in host_config.commands.iter() {
                let command_spec = ModuleSpecification::new(command_id, &command_config.version);
                let command = match self.module_factory.new_command(&command_spec, &command_config.settings) {
                    Some(command) => command,
                    None => continue,
                };

                if let Some(connector_spec) = command.get_connector_spec() {
                    let connector_settings = match host_config.connectors.get(&connector_spec.id) {
                        Some(config) => config.settings.clone(),
                        None => HashMap::new(),
                    };

                    let connector = self.module_factory.new_connector(&connector_spec, &connector_settings);
                    if !connector.get_metadata_self().is_stateless {
                        host_connectors.entry(connector_spec).or_insert_with(|| Arc::new(Mutex::new(connector)));
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
        self.new_request_sender()
            .send(ConnectorRequest::exit_token())
            .unwrap_or_else(|error| log::error!("Couldn't send exit token to connection manager: {}", error));

        self.join();
    }

    pub fn join(&mut self) {
        if let Some(thread) = self.receiver_thread.take() {
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
        mut stateful_connectors: HashMap<String, ConnectorStates>,
        receiver: mpsc::Receiver<ConnectorRequest>,
        module_factory: Arc<ModuleFactory>,
        cache_settings: CacheSettings) -> thread::JoinHandle<()> {

        thread::spawn(move || {
            let worker_pool = rayon::ThreadPoolBuilder::new().num_threads(MAX_WORKER_THREADS).build().unwrap();
            log::debug!("Created worker pool with {} threads", MAX_WORKER_THREADS);

            let command_cache = Arc::new(Mutex::new(Self::initialize_cache(cache_settings.clone())));

            loop {
                let request = match receiver.recv() {
                    Ok(data) => data,
                    Err(error) => {
                        log::error!("Stopped receiver thread: {}", error);
                        return;
                    }
                };

                if let RequestType::Exit = request.request_type {
                    log::debug!("Gracefully exiting connection manager thread");

                    if cache_settings.enable_cache {
                        match command_cache.lock().unwrap().write_to_disk() {
                            Ok(count) => log::debug!("Wrote {} entries to cache file", count),
                            // Failing to write the file is not critical.
                            Err(error) => log::error!("{}", error),
                        }
                    }
                    return;
                }

                // Requests with no connector dependency.
                if request.connector_spec.is_none() {
                    // (request.response_handler)(RequestResponse::new_empty(request.source_id, request.host, request.invocation_id));
                    continue;
                }

                let mut connector_spec = request.connector_spec.as_ref().unwrap().clone();
                connector_spec.module_type = String::from("connector");
                let connector_metadata = module_factory.get_connector_module_metadata(&connector_spec);

                // Stateless connectors.
                if connector_metadata.is_stateless {
                    worker_pool.install(|| {
                        log::debug!("[{}] Worker {} processing a stateless request", request.host.name, rayon::current_thread_index().unwrap());
                        let mut connector = module_factory.new_connector(&connector_spec, &HashMap::new());
                        let responses = match &request.request_type {
                            RequestType::Command { cache_policy, commands } => {
                                commands.par_iter().map(|command| {
                                    let mut par_connector = module_factory.new_connector(&connector_spec, &HashMap::new());
                                    Self::process_command(&request.host, command_cache.clone(), cache_policy, &mut par_connector, command )
                                }).collect()
                            },
                            RequestType::CommandFollowOutput { commands: _ } =>
                                panic!("Follow output is currently not supported for stateless connectors"),
                            RequestType::Download { remote_file_path: file_path } =>
                                vec![Self::process_download(&request.host, &mut connector, &file_path)],
                            RequestType::Upload { metadata, local_file_path } =>
                                vec![Self::process_upload(&request.host, &mut connector, local_file_path, metadata)],
                            // Exit is handled earlier.
                            RequestType::Exit => panic!(),
                        };

                        let response = RequestResponse::new(&request, responses);
                        // (request.response_handler)(response);
                    });
                }
                // Stateful connectors.
                else {
                    // TODO: This will block the thread unnecessarily. Need a better solution for parallel update of multiple hosts.
                    // Imagine MAX_WORKERS amount of requests sequentially for the same host and connector.
                    let host_connectors = stateful_connectors.get_mut(&request.host.name).unwrap();
                    let connector_mutex = host_connectors.get_mut(&connector_spec).unwrap();

                    let mut connector = connector_mutex.lock().unwrap();
                    if !connector.is_connected() {
                        if let Err(error) = connector.connect(&request.host.ip_address) {
                            log::error!("[{}] Error while connecting {}: {}", request.host.name, request.host.ip_address, error);
                            let response = RequestResponse::new_error(request.source_id, request.host, request.invocation_id, format!("Error while connecting: {}", error));
                            // (request.response_handler)(response);
                            continue;
                        }
                    }
                    drop(connector);

                    worker_pool.install(|| {
                        log::debug!("[{}] Worker {} processing a stateful request", request.host.name, rayon::current_thread_index().unwrap());

                        let mut connector = connector_mutex.lock().unwrap();
                        let responses = match &request.request_type {
                            RequestType::Command { cache_policy, commands } => {
                                commands.iter().map(|command| {
                                    Self::process_command(&request.host, command_cache.clone(), cache_policy, &mut connector, command )
                                }).collect()
                            },
                            RequestType::CommandFollowOutput { commands } => {
                                if commands.len() != 1 {
                                    panic!("Follow output is only supported for a single command");
                                }
                                vec![Self::process_command_follow_output(&request.host, &mut connector, &commands.first().unwrap())]
                            },
                            RequestType::Download { remote_file_path: file_path } =>
                                vec![Self::process_download(&request.host, &mut connector, &file_path)],
                            RequestType::Upload { metadata, local_file_path } =>
                                vec![Self::process_upload(&request.host, &mut connector, local_file_path, metadata)],
                            // Exit is handled earlier.
                            RequestType::Exit => panic!(),
                        };
                        drop(connector);

                        let response = RequestResponse::new(&request, responses);
                        // (request.response_handler)(response);
                    });
                }
            }
        })
    }

    fn process_command(host: &Host,
                       command_cache: Arc<Mutex<Cache<String, ResponseMessage>>>,
                       cache_policy: &CachePolicy,
                       connector: &mut Connector,
                       request_message: &String) -> Result<ResponseMessage, String> {

        let mut command_cache = command_cache.lock().unwrap();

        // let request = request.lock().unwrap();
        log::debug!("[{}] Processing command: {}", host.name, request_message);

        // Some commands are supposed to not actually execute.
        if request_message.is_empty() {
            return Ok(ResponseMessage::empty());
        }

        let cache_key = match connector.get_metadata_self().cache_scope {
            CacheScope::Global => format!("{}|{}", connector.get_module_spec(), request_message),
            CacheScope::Host => format!("{}|{}|{}", host.name, connector.get_module_spec(), request_message),
        };

        let cached_response = if *cache_policy == CachePolicy::OnlyCache || *cache_policy == CachePolicy::PreferCache {
            command_cache.get(&cache_key)
        }
        else {
            None
        };

        if let Some(cached_response) = cached_response {
            log::debug!("[{}] Using cached response for command {}", host.name, request_message);
            return Ok(cached_response);
        }
        else {
            if *cache_policy == CachePolicy::OnlyCache {
                return Ok(ResponseMessage::not_found());
            }

            let response_result = connector.send_message(&request_message, true);

            if let Ok(response) = response_result {
                if response.return_code != 0 {
                    log::debug!("Command returned non-zero exit code: {}", response.return_code)
                }
                else {
                    // Doesn't cache failed commands.
                    let mut cached_response = response.clone();
                    cached_response.is_from_cache = true;
                    command_cache.insert(cache_key, cached_response);
                }
                Ok(response)
            }
            else {
                response_result
            }
        }
    }

    fn process_command_follow_output(host: &Host, connector: &mut Connector, request_message: &String) -> Result<ResponseMessage, String> {
        log::debug!("[{}] Processing command: {}", host.name, request_message);
        let response_result = connector.send_message(&request_message, false);

        if let Ok(response) = response_result {
            if response.return_code != 0 {
                log::debug!("Command returned non-zero exit code: {}", response.return_code)
            }
            Ok(response)
        }
        else {
            response_result
        }
    }

    fn process_download(host: &Host, connector: &mut Connector, file_path: &String) -> Result<ResponseMessage, String> {
        log::debug!("[{}] Downloading file: {}", host.name, file_path);
        match connector.download_file(&file_path) {
            Ok((metadata, contents)) => {
                match file_handler::create_file(&host, &file_path, metadata, contents) {
                    Ok(file_path) => Ok(ResponseMessage::new_success(file_path)),
                    Err(error) => Err(error.to_string()),
                }
            },
            Err(error) => Err(error.to_string()),
        }
    }

    fn process_upload(host: &Host, connector: &mut Connector, local_file_path: &String, file_metadata: &FileMetadata) -> Result<ResponseMessage, String> {
        log::debug!("[{}] Uploading file: {}", host.name, local_file_path);
        match file_handler::read_file(&local_file_path) {
            Ok((metadata, contents)) => {
                let result = connector.upload_file(&metadata, contents);

                match result {
                    Ok(_) => Ok(ResponseMessage::empty()),
                    Err(error) => Err(error.to_string()),
                }
            },
            Err(error) => Err(error.to_string()),
        }
    }
}

pub struct ConnectorRequest {
    pub connector_spec: Option<ModuleSpecification>,
    pub source_id: String,
    pub host: Host,
    pub invocation_id: u64,
    pub request_type: RequestType,
}

impl ConnectorRequest {
    pub fn exit_token() -> Self {
        ConnectorRequest {
            connector_spec: None,
            source_id: String::new(),
            host: Host::default(),
            invocation_id: 0,
            request_type: RequestType::Exit,
        }
    }
}

impl Debug for ConnectorRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ConnectorRequest {:?}]", self.connector_spec)
    }
}

#[derive(Debug, Clone)]
pub enum RequestType {
    Command {
        cache_policy: CachePolicy,
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
    /// Causes the receiver thread to exit.
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CachePolicy {
    BypassCache,
    PreferCache,
    OnlyCache,
}