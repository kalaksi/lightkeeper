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
use crate::file_handler;
use crate::module::ModuleFactory;
use crate::module::ModuleSpecification;
use crate::module::connection::*;
use crate::cache::{Cache, CacheScope};

pub type ResponseHandlerCallback = Box<dyn FnOnce(Vec<Result<ResponseMessage, String>>) + Send + 'static>;
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
                let command = self.module_factory.new_command(&command_spec, &command_config.settings);

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

                if request.request_type == RequestType::Exit {
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
                    (request.response_handler)(Vec::new());
                }
                else {
                    let mut connector_spec = request.connector_spec.as_ref().unwrap().clone();
                    connector_spec.module_type = String::from("connector");
                    let connector_metadata = module_factory.get_connector_module_metadata(&connector_spec);
                    let request_messages = request.messages.clone();

                    // Stateless connectors.
                    if connector_metadata.is_stateless {
                        // Wrapping to Arc<Mutex<>> to allow passing to Self::process_request.
                        let mutex_request = Arc::new(Mutex::new(request));

                        worker_pool.install(|| {
                            let responses = request_messages.par_iter().map(|request_message| {
                                if request_message.is_empty() {
                                    return Ok(ResponseMessage::empty());
                                }

                                log::debug!("Worker {} processing a stateless request", rayon::current_thread_index().unwrap());
                                let connector = Arc::new(Mutex::new(module_factory.new_connector(&connector_spec, &HashMap::new())));
                                Self::process_request(mutex_request.clone(), request_message, connector.clone(), command_cache.clone())
                            }).collect();

                            let request = Arc::try_unwrap(mutex_request).unwrap().into_inner().unwrap();
                            (request.response_handler)(responses);
                        });
                    }
                    // Stateful connectors.
                    else {
                        // TODO: This will block the thread unnecessarily. Need better solution. Imagine
                        // MAX_WORKERS amount of requests sequentially for the same host and connector.
                        let host_connectors = stateful_connectors.get_mut(&request.host.name).unwrap();
                        let connector_mutex = host_connectors.get_mut(&connector_spec).unwrap();

                        let mut connector = connector_mutex.lock().unwrap();
                        if !connector.is_connected() {
                            if let Err(error) = connector.connect(&request.host.ip_address) {
                                log::error!("[{}] Error while connecting {}: {}", request.host.name, request.host.ip_address, error);
                                (request.response_handler)(vec![Err(format!("Error while connecting: {}", error))]);
                                continue;
                            }
                        }
                        drop(connector);

                        let request_mutex = Arc::new(Mutex::new(request));
                        worker_pool.install(|| {
                            let responses = request_messages.iter().map(|request_message| {
                                if request_message.is_empty() {
                                    return Ok(ResponseMessage::empty());
                                }

                                log::debug!("Worker {} processing a stateful request", rayon::current_thread_index().unwrap());
                                Self::process_request(request_mutex.clone(), request_message, connector_mutex.clone(), command_cache.clone())
                            }).collect();

                            let request = Arc::try_unwrap(request_mutex).unwrap().into_inner().unwrap();
                            (request.response_handler)(responses);
                        });
                    }
                }
            }
        })
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


    fn process_request(request: Arc<Mutex<ConnectorRequest>>, request_message: &String, connector: Arc<Mutex<Connector>>,
                       command_cache: Arc<Mutex<Cache<String, ResponseMessage>>>) -> Result<ResponseMessage, String> { 

        let request = request.lock().unwrap();
        let mut connector = connector.lock().unwrap();
        let mut command_cache = command_cache.lock().unwrap();

        match &request.request_type {
            RequestType::Command => {
                log::debug!("[{}] Processing command: {}", request.host.name, request_message);

                let cache_key = match connector.get_metadata_self().cache_scope {
                    CacheScope::Global => format!("{}|{}", connector.get_module_spec(), request_message),
                    CacheScope::Host => format!("{}|{}|{}", request.host.name, connector.get_module_spec(), request_message),
                };

                let cached_response = if request.cache_policy == CachePolicy::OnlyCache || request.cache_policy == CachePolicy::PreferCache {
                    command_cache.get(&cache_key)
                }
                else {
                    None
                };

                if let Some(cached_response) = cached_response {
                    log::debug!("[{}] Using cached response for command {}", request.host.name, request_message);
                    return Ok(cached_response);
                }
                else {
                    if request.cache_policy == CachePolicy::OnlyCache {
                        return Ok(ResponseMessage::not_found());
                    }

                    let response_result = connector.send_message(request_message);

                    if response_result.is_ok() {
                        let response = response_result.as_ref().unwrap().clone();
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
            },
            RequestType::Download => {
                log::debug!("[{}] Downloading file: {}", request.host.name, request_message);
                match connector.download_file(request_message) {
                    Ok((metadata, contents)) => {
                        match file_handler::create_file(&request.host, request_message, metadata, contents) {
                            Ok(file_path) => Ok(ResponseMessage::new_success(file_path)),
                            Err(error) => Err(error.to_string()),
                        }
                    },
                    Err(error) => Err(error.to_string()),
                }
            },
            RequestType::Upload => {
                let local_file_path = request_message;
                log::debug!("[{}] Uploading file: {}", request.host.name, local_file_path);
                match file_handler::read_file(local_file_path) {
                    Ok((metadata, contents)) => {
                        let local_file_hash = sha256::digest(contents.as_slice());

                        let mut result = Ok(());
                        let mut response_message = String::new();
                        // Only upload if contents changed.
                        if local_file_hash != metadata.remote_file_hash {
                            result = connector.upload_file(&metadata, contents);
                            if result.is_ok() {
                                response_message = String::from("File updated");
                            }
                        }
                        else {
                            response_message = String::from("File unchanged");
                            log::info!("{}", response_message);
                        }

                        if metadata.temporary {
                            log::debug!("Removing temporary local file {}", local_file_path);
                            if let Err(error) = file_handler::remove_file(local_file_path) {
                                log::error!("Error while removing: {}", error);
                            }
                        }

                        if let Err(error) = result {
                            Err(error.to_string())
                        }
                        else {
                            Ok(ResponseMessage::new(response_message, 0))
                        }
                    },
                    Err(error) => Err(error.to_string()),
                }
            },
            // Exit is handled earlier.
            RequestType::Exit => panic!(),
        }
    }
}

pub struct ConnectorRequest {
    pub connector_spec: Option<ModuleSpecification>,
    pub source_id: String,
    pub host: Host,
    pub messages: Vec<String>,
    pub request_type: RequestType,
    pub response_handler: ResponseHandlerCallback,
    pub cache_policy: CachePolicy,
}

impl ConnectorRequest {
    pub fn exit_token() -> Self {
        ConnectorRequest {
            connector_spec: None,
            source_id: String::new(),
            host: Host::new(&String::new(), &String::from("127.0.0.1"), &String::new(), &Vec::new()).unwrap(),
            messages: Vec::new(),
            request_type: RequestType::Exit,
            response_handler: Box::new(|_| ()),
            cache_policy: CachePolicy::BypassCache,
        }
    }
}

impl Debug for ConnectorRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ConnectorRequest {:?}]", self.connector_spec)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RequestType {
    Command,
    Download,
    Upload,
    Exit,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CachePolicy {
    BypassCache,
    PreferCache,
    OnlyCache,
}